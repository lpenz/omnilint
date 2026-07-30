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
mod linters;

use clap::Parser;
use std::error::Error;
use tokio_stream::StreamExt;

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
            for file in &files {
                eprintln!("Analyzing: {}", file.display());
                let ext = file.extension().and_then(|e| e.to_str());
                match ext {
                    Some("yaml" | "yml") => {
                        let mut yamllint = linters::yamllint::YamlYamllint::new(file)?;
                        while let Some(entry) = yamllint.next().await {
                            eprintln!("{}", entry);
                        }
                    }
                    Some("py") => {
                        let mut flake8 = linters::flake8::PythonFlake8::new(file)?;
                        while let Some(entry) = flake8.next().await {
                            eprintln!("{}", entry);
                        }
                    }
                    Some("sh" | "bash" | "dash" | "ksh") => {
                        let mut shellcheck = linters::shellcheck::ShShellcheck::new(file)?;
                        while let Some(entry) = shellcheck.next().await {
                            eprintln!("{}", entry);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
