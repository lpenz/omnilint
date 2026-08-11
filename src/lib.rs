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

mod entry;
mod filetype;
mod linters;
mod repo;

use crate::entry::Entry;
use crate::filetype::Filetype;

use clap::Parser;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use tokio_stream::{Stream, StreamExt, StreamMap};

/// main function, the single pub function in this lib.
#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ACTIVE)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let args = cli::Cli::parse();
    match args.command {
        cli::Commands::Files { files } => {
            let mut streams: Vec<Pin<Box<dyn Stream<Item = Entry>>>> = Vec::new();
            for file in &files {
                push_stream(&mut streams, file)?;
            }
            run_streams(streams).await;
        }
        cli::Commands::Repository => {
            let files = repo::git_ls_files()?;
            run_repository(files).await?;
        }
    }
    Ok(())
}

/// Creates a stream that lints the given file, or `None` if there is no
/// linter for its [`Filetype`].
fn stream_for_file(file: &Path) -> color_eyre::Result<Option<Pin<Box<dyn Stream<Item = Entry>>>>> {
    let filetype = Filetype::detect(file);
    let stream: Pin<Box<dyn Stream<Item = Entry>>> = match filetype {
        Filetype::Yaml => Box::pin(linters::yamllint::YamlYamllint::new(file)?),
        Filetype::Python => {
            let flake8 = linters::flake8::PythonFlake8::new(file)?;
            let ruff = linters::ruff::PythonRuff::new(file)?;
            Box::pin(flake8.merge(ruff))
        }
        Filetype::Shell => Box::pin(linters::shellcheck::ShShellcheck::new(file)?),
        _ => return Ok(None),
    };
    Ok(Some(stream))
}

/// Creates a stream that lints the given file and pushes it into `streams`,
/// if there is a linter for its [`Filetype`].
fn push_stream(
    streams: &mut Vec<Pin<Box<dyn Stream<Item = Entry>>>>,
    file: &Path,
) -> color_eyre::Result<()> {
    if let Some(stream) = stream_for_file(file)? {
        streams.push(stream);
    }
    Ok(())
}

/// Lints each file of the given stream as soon as it is produced, running
/// all the linters in parallel, and prints the resulting [`Entry`] values to
/// stderr.
async fn run_repository(files: impl Stream<Item = PathBuf> + Unpin) -> color_eyre::Result<()> {
    let mut files = files;
    let mut linters: StreamMap<usize, Pin<Box<dyn Stream<Item = Entry>>>> = StreamMap::new();
    let mut next_id = 0;
    loop {
        if linters.is_empty() {
            match files.next().await {
                Some(file) => add_file(&mut linters, &mut next_id, &file)?,
                None => break,
            }
        } else {
            tokio::select! {
                maybe_file = files.next() => {
                    match maybe_file {
                        Some(file) => add_file(&mut linters, &mut next_id, &file)?,
                        None => break,
                    }
                }
                Some((_, entry)) = linters.next() => {
                    eprintln!("{}", entry);
                }
            }
        }
    }
    while let Some((_, entry)) = linters.next().await {
        eprintln!("{}", entry);
    }
    Ok(())
}

/// Creates a linter stream for `file`, if any, and adds it to `linters`.
fn add_file(
    linters: &mut StreamMap<usize, Pin<Box<dyn Stream<Item = Entry>>>>,
    next_id: &mut usize,
    file: &Path,
) -> color_eyre::Result<()> {
    if let Some(stream) = stream_for_file(file)? {
        linters.insert(*next_id, stream);
        *next_id += 1;
    }
    Ok(())
}

/// Lints all the given streams in parallel, printing the resulting
/// [`Entry`] values to stderr.
async fn run_streams(streams: Vec<Pin<Box<dyn Stream<Item = Entry>>>>) {
    let merged = streams.into_iter().reduce(|a, b| Box::pin(a.merge(b)));
    if let Some(mut merged) = merged {
        while let Some(entry) = merged.next().await {
            eprintln!("{}", entry);
        }
    }
}
