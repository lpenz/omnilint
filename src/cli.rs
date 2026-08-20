// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

// CLI using [`clap`]
//
// This is not a module-level doc because we `include!` it in build.rs.
//
// [`clap`]: https://docs.rs/clap/latest/clap/

use clap::{Parser, Subcommand, ValueEnum};
use serde::Deserialize;
use std::fmt;
use std::path::PathBuf;

/// Controls what happens when a linter binary is not found on the `PATH`.
#[derive(Debug, Default, Clone, Copy, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "lowercase")]
#[value(rename_all = "lowercase")]
pub enum LinterMode {
    /// Abort with an error when the linter is not found.
    Required,
    /// Emit an entry when the linter is not found (counts as an issue).
    #[default]
    Wanted,
    /// Run the linter if available, silently skip when not found.
    Optional,
    /// Don't run the linter at all, even if the binary exists.
    Disabled,
}

impl fmt::Display for LinterMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinterMode::Required => write!(f, "required"),
            LinterMode::Wanted => write!(f, "wanted"),
            LinterMode::Optional => write!(f, "optional"),
            LinterMode::Disabled => write!(f, "disabled"),
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "Statically analyse any file with the appropriate tools"
)]
pub struct Cli {
    /// Default mode for linters that are not found on the `PATH`:
    /// `required` (abort), `wanted` (emit entry), `optional` (skip),
    /// or `disabled` (don't run).
    #[arg(long, value_enum, default_value_t, global = true)]
    pub default_linter_mode: LinterMode,

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
    fn default_linter_mode_flag() {
        let cli = Cli::try_parse_from(["", "files", "--default-linter-mode", "optional", "foo.py"])
            .unwrap();
        assert_eq!(cli.default_linter_mode, LinterMode::Optional);
        let cli = Cli::try_parse_from(["", "files", "--default-linter-mode", "disabled", "foo.py"])
            .unwrap();
        assert_eq!(cli.default_linter_mode, LinterMode::Disabled);
        let cli = Cli::try_parse_from(["", "files", "--default-linter-mode", "required", "foo.py"])
            .unwrap();
        assert_eq!(cli.default_linter_mode, LinterMode::Required);
        let cli = Cli::try_parse_from(["", "files", "foo.py"]).unwrap();
        assert_eq!(cli.default_linter_mode, LinterMode::Wanted);
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
