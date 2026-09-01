// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Statically analyse any file with the appropriate tools

#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![allow(rustdoc::private_intra_doc_links)]

mod cli;

mod config;
mod entry;
mod error;
mod filetype;
mod linters;
mod repo;

use crate::cli::LinterMode;
use crate::entry::Entry;
use crate::error::OmnilintError;
use crate::linters::{ALL_LINTERS, Linters, is_builtin};

use clap::Parser;
use cli::OutputFormat;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use tokio_stream::{Stream, StreamExt, StreamMap};

/// main function, the single pub function in this lib.
///
/// Exits with status 1 if any finding was emitted (including a linter that
/// was not found, or a missing required linter in the inventory), printing
/// the reason to stderr, and with status 0 otherwise.
#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ACTIVE)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let args = cli::Cli::parse();
    let mut linters = Linters::new();
    let config = config::Config::load(args.config.as_deref())?;
    let default_mode = if args.default_linter_mode != LinterMode::default() {
        args.default_linter_mode
    } else {
        config.global.default_linter_mode
    };
    linters.set_default_mode(default_mode);
    let mode_overrides: std::collections::HashMap<String, LinterMode> = config
        .linters
        .iter()
        .filter_map(|(name, c)| c.mode.map(|mode| (name.clone(), mode)))
        .collect();
    linters.set_mode_overrides(mode_overrides);
    let paths: std::collections::HashMap<String, String> = config
        .linters
        .iter()
        .filter_map(|(name, c)| c.path.as_ref().map(|p| (name.clone(), p.clone())))
        .collect();
    linters.set_executables(paths);
    let format = args.format;
    let run_result: Result<(), OmnilintError> = match args.command {
        cli::Commands::Files { files } => {
            let mut streams: Vec<Pin<Box<dyn Stream<Item = Entry>>>> = Vec::new();
            for file in &files {
                push_stream(&mut linters, &mut streams, file)?;
            }
            run_streams(streams, format).await
        }
        cli::Commands::Repository => {
            let files = repo::git_ls_files()?;
            run_repository(&mut linters, files, format).await?
        }
        cli::Commands::Inventory => run_inventory(&linters).await,
    };
    if let Err(error) = run_result {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
    Ok(())
}

/// Shows the status of all supported linters: their mode and version
/// when available.
///
/// Prints a message to stderr for each [`LinterMode::Required`] linter that
/// was not found, and returns [`OmnilintError::MissingRequiredLinters`] if
/// any were missing.
async fn run_inventory(linters: &Linters) -> Result<(), OmnilintError> {
    let mut missing_required = 0;
    for &name in ALL_LINTERS {
        let mode = linters.resolve_mode(name);
        let version = if is_builtin(name) {
            "built-in".to_string()
        } else {
            let executable = linters.executable_for_linter(name);
            get_version(executable.as_ref()).await
        };
        if mode == LinterMode::Required && version == "not found" {
            eprintln!("error: required linter '{name}' not found");
            missing_required += 1;
        }
        let mode = mode.to_string();
        eprintln!("{name:<20} {mode:<11} {version}");
    }
    if missing_required > 0 {
        Err(OmnilintError::MissingRequiredLinters)
    } else {
        Ok(())
    }
}

/// Runs `executable` with `arg`, returning the output if it could be
/// spawned, or `None` if the executable doesn't exist.
async fn run_version(executable: &str, arg: &str) -> Option<std::process::Output> {
    tokio::process::Command::new(executable)
        .arg(arg)
        .output()
        .await
        .ok()
}

/// Tries to get the version string of an executable by running it with
/// `--version`, falling back to `version` for tools that don't accept that
/// flag (e.g. go). Returns "not found" if the executable doesn't exist.
async fn get_version(executable: &str) -> String {
    let output = match run_version(executable, "--version").await {
        Some(output) if output.status.success() => output,
        _ => match run_version(executable, "version").await {
            Some(output) => output,
            None => return "not found".to_string(),
        },
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let raw = if !stdout.is_empty() {
        stdout.trim()
    } else {
        stderr.trim()
    };
    // Strip trailing newlines and take the first line
    raw.lines().next().unwrap_or(raw).to_string()
}

/// Creates a stream that lints the given file and pushes it into `streams`,
/// if there is a linter for its file type.
fn push_stream(
    linters: &mut Linters,
    streams: &mut Vec<Pin<Box<dyn Stream<Item = Entry>>>>,
    file: &Path,
) -> color_eyre::Result<()> {
    if let Some(stream) = linters.stream_for_file(file)? {
        streams.push(stream);
    }
    Ok(())
}

/// Lints each file of the given stream as soon as it is produced, running
/// all the linters in parallel, and prints the resulting [`Entry`] values to
/// stderr. Returns [`OmnilintError::Findings`] if any entry was emitted, or
/// a runtime error from the linter infrastructure.
async fn run_repository(
    linters: &mut Linters,
    files: impl Stream<Item = PathBuf> + Unpin,
    format: OutputFormat,
) -> color_eyre::Result<Result<(), OmnilintError>> {
    let mut files = files;
    let mut streams: StreamMap<usize, Pin<Box<dyn Stream<Item = Entry>>>> = StreamMap::new();
    let mut next_id = 0;
    let mut issues = 0;
    loop {
        if streams.is_empty() {
            match files.next().await {
                Some(file) => add_file(linters, &mut streams, &mut next_id, &file)?,
                None => break,
            }
        } else {
            tokio::select! {
                maybe_file = files.next() => {
                    match maybe_file {
                        Some(file) => add_file(linters, &mut streams, &mut next_id, &file)?,
                        None => break,
                    }
                }
                Some((_, entry)) = streams.next() => {
                    eprintln!("{}", entry.format_output(format));
                    issues += 1;
                }
            }
        }
    }
    while let Some((_, entry)) = streams.next().await {
        eprintln!("{}", entry.format_output(format));
        issues += 1;
    }
    if issues > 0 {
        Ok(Err(OmnilintError::Findings))
    } else {
        Ok(Ok(()))
    }
}

/// Creates a linter stream for `file`, if any, and adds it to `streams`.
fn add_file(
    linters: &mut Linters,
    streams: &mut StreamMap<usize, Pin<Box<dyn Stream<Item = Entry>>>>,
    next_id: &mut usize,
    file: &Path,
) -> color_eyre::Result<()> {
    if let Some(stream) = linters.stream_for_file(file)? {
        streams.insert(*next_id, stream);
        *next_id += 1;
    }
    Ok(())
}

/// Lints all the given streams in parallel, printing the resulting
/// [`Entry`] values to stderr. Returns [`OmnilintError::Findings`] if any
/// entry was emitted.
async fn run_streams(
    streams: Vec<Pin<Box<dyn Stream<Item = Entry>>>>,
    format: OutputFormat,
) -> Result<(), OmnilintError> {
    let merged = streams.into_iter().reduce(|a, b| Box::pin(a.merge(b)));
    let mut issues = 0;
    if let Some(mut merged) = merged {
        while let Some(entry) = merged.next().await {
            eprintln!("{}", entry.format_output(format));
            issues += 1;
        }
    }
    if issues > 0 {
        Err(OmnilintError::Findings)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
