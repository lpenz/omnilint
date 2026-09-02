// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [actionlint](https://github.com/rhysd/actionlint) GitHub Actions workflow
//! linter wrapper.
//!
//! actionlint checks GitHub Actions workflow files for incorrect or
//! deprecated syntax. It is run once per file with a custom output template
//! and its output is parsed into [`Entry`] values.
//!
//! ## Output format
//!
//! The `-format` template makes actionlint emit each finding on stdout as:
//!
//! ```text
//! <filename>:<line>:<col>: <message>
//! ```
//!
//! For example:
//!
//! ```text
//! .github/workflows/ci.yml:8:9: step must run script with "run" section or run action with "uses" section
//! ```
//!
//! The template iterates over the slice of errors, so it must use `range`.

use crate::entry::Entry;
use crate::linters::{CommandLinter, Linters, Spec, parse_line_standard};

use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio_stream::Stream;

pub struct GithubWorkflowActionlint(CommandLinter);

impl GithubWorkflowActionlint {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        Ok(Self(CommandLinter::new(
            linters,
            Spec {
                name: "actionlint",
                args: &[
                    "-no-color",
                    "-format",
                    "{{range .}}{{.Filepath}}:{{.Line}}:{{.Column}}: {{.Message}}\\n{{end}}",
                ],
                parse: parse_line,
                ..Default::default()
            },
            filename,
        )?))
    }
}

fn parse_line(filename: &Path, line: &str) -> Vec<Entry> {
    parse_line_standard(filename, "actionlint", line)
}

linter_stream!(GithubWorkflowActionlint);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_standard() {
        let entries = parse_line(
            Path::new(".github/workflows/ci.yml"),
            ".github/workflows/ci.yml:8:9: step must run script with \"run\" section or run action with \"uses\" section",
        );
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].to_string(),
            ".github/workflows/ci.yml:8: [actionlint] step must run script with \"run\" section or run action with \"uses\" section"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(parse_line(Path::new("x.yml"), "").is_empty());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(parse_line(Path::new("x.yml"), "x.yml:x:y: msg").is_empty());
    }
}
