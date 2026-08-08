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

use crate::entry::Entry;
use crate::filetype::Filetype;

use clap::Parser;
use std::error::Error;
use std::pin::Pin;
use tokio_stream::{Stream, StreamExt};

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
                let filetype = Filetype::detect(file);
                let stream: Pin<Box<dyn Stream<Item = Entry>>> = match filetype {
                    Filetype::Yaml => Box::pin(linters::yamllint::YamlYamllint::new(file)?),
                    Filetype::Python => {
                        let flake8 = linters::flake8::PythonFlake8::new(file)?;
                        let ruff = linters::ruff::PythonRuff::new(file)?;
                        Box::pin(flake8.merge(ruff))
                    }
                    Filetype::Shell => Box::pin(linters::shellcheck::ShShellcheck::new(file)?),
                    _ => continue,
                };
                streams.push(stream);
            }
            let merged = streams.into_iter().reduce(|a, b| Box::pin(a.merge(b)));
            if let Some(mut merged) = merged {
                while let Some(entry) = merged.next().await {
                    eprintln!("{}", entry);
                }
            }
        }
    }
    Ok(())
}
