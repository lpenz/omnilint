// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

// CLI using [`clap`]
//
// This is not a module-level doc because we `include!` it in build.rs.
//
// [`clap`]: https://docs.rs/clap/latest/clap/

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "Statically analyse any file with the appropriate tools"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Analyse the given files with the appropriate tools
    Files {
        /// The files to analyse
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
    /// Analyse all the files tracked by git in the current repository
    Repository,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let cli = Cli::try_parse_from([""]);
        assert!(cli.is_err());
    }

    #[test]
    fn files_basic() {
        let cli = Cli::try_parse_from(["", "files", "foo.rs", "bar.rs"]).unwrap();
        let Commands::Files { files } = cli.command else {
            unreachable!("expected files command");
        };
        assert_eq!(
            files,
            vec![PathBuf::from("foo.rs"), PathBuf::from("bar.rs")]
        );
    }

    #[test]
    fn files_requires_args() {
        let cli = Cli::try_parse_from(["", "files"]);
        assert!(cli.is_err());
    }

    #[test]
    fn repository_basic() {
        let cli = Cli::try_parse_from(["", "repository"]).unwrap();
        assert!(matches!(cli.command, Commands::Repository));
    }
}
