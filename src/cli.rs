// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

// CLI using [`clap`]
//
// This is not a module-level doc because we `include!` it in build.rs.
//
// [`clap`]: https://docs.rs/clap/latest/clap/

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "Statically analyse any file with the appropriate tools"
)]
pub struct Cli {
    /// Ignore linters that are not found on the `PATH`: don't report them and
    /// don't consider them an issue for the exit status. Can also be enabled
    /// by setting the `OMNILINT_IGNORE_MISSING_LINTERS` environment variable
    /// to a truthy value (`1`, `true`, `yes` or `on`).
    #[arg(long, global = true)]
    pub ignore_missing_linters: bool,

    /// Output format
    #[arg(long, value_enum, default_value_t, global = true)]
    pub format: OutputFormat,

    #[command(subcommand)]
    pub command: Commands,
}

/// Output format for lint results.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Default human-readable format: `file:line: [linter] message`
    #[default]
    Default,

    /// GitHub Actions workflow commands: `::warning file=...,line=...,...::message`
    GithubWorkflow,
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
    /// Show the status of all supported linters
    Inventory,
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
    fn ignore_missing_linters_flag() {
        let cli = Cli::try_parse_from(["", "files", "--ignore-missing-linters", "foo.py"]).unwrap();
        assert!(cli.ignore_missing_linters);
        let cli = Cli::try_parse_from(["", "files", "foo.py", "--ignore-missing-linters"]).unwrap();
        assert!(cli.ignore_missing_linters);
        let cli = Cli::try_parse_from(["", "repository", "--ignore-missing-linters"]).unwrap();
        assert!(cli.ignore_missing_linters);
        let cli = Cli::try_parse_from(["", "files", "foo.py"]).unwrap();
        assert!(!cli.ignore_missing_linters);
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

    #[test]
    fn format_default() {
        let cli = Cli::try_parse_from(["", "files", "foo.py"]).unwrap();
        assert_eq!(cli.format, OutputFormat::Default);
    }

    #[test]
    fn format_github_workflow() {
        let cli =
            Cli::try_parse_from(["", "files", "--format", "github-workflow", "foo.py"]).unwrap();
        assert_eq!(cli.format, OutputFormat::GithubWorkflow);
    }

    #[test]
    fn inventory_basic() {
        let cli = Cli::try_parse_from(["", "inventory"]).unwrap();
        assert!(matches!(cli.command, Commands::Inventory));
    }
}
