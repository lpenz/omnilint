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
use crate::linters::Linters;

use clap::Parser;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use tokio_stream::{Stream, StreamExt, StreamMap};

/// main function, the single pub function in this lib.
///
/// Exits with status 1 if any finding was emitted (including a linter that
/// was not found), and with status 0 otherwise.
#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ACTIVE)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let args = cli::Cli::parse();
    let mut linters = Linters::new();
    linters.set_ignore_missing(args.ignore_missing_linters);
    let issues = match args.command {
        cli::Commands::Files { files } => {
            let mut streams: Vec<Pin<Box<dyn Stream<Item = Entry>>>> = Vec::new();
            for file in &files {
                push_stream(&mut linters, &mut streams, file)?;
            }
            run_streams(streams).await
        }
        cli::Commands::Repository => {
            let files = repo::git_ls_files()?;
            run_repository(&mut linters, files).await?
        }
    };
    if issues > 0 {
        std::process::exit(1);
    }
    Ok(())
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
/// stderr. Returns the number of entries emitted.
async fn run_repository(
    linters: &mut Linters,
    files: impl Stream<Item = PathBuf> + Unpin,
) -> color_eyre::Result<usize> {
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
                    eprintln!("{}", entry);
                    issues += 1;
                }
            }
        }
    }
    while let Some((_, entry)) = streams.next().await {
        eprintln!("{}", entry);
        issues += 1;
    }
    Ok(issues)
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
/// [`Entry`] values to stderr. Returns the number of entries emitted.
async fn run_streams(streams: Vec<Pin<Box<dyn Stream<Item = Entry>>>>) -> usize {
    let merged = streams.into_iter().reduce(|a, b| Box::pin(a.merge(b)));
    let mut issues = 0;
    if let Some(mut merged) = merged {
        while let Some(entry) = merged.next().await {
            eprintln!("{}", entry);
            issues += 1;
        }
    }
    issues
}
