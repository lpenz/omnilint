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
use crate::linters::{Linter, Linters, parse_line_standard};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct GithubWorkflowActionlint {
    filename: PathBuf,
    inner: Linter,
}

impl GithubWorkflowActionlint {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let mut cmd = Command::new("actionlint");
        cmd.arg("-no-color");
        cmd.arg("-format");
        cmd.arg("{{range .}}{{.Filepath}}:{{.Line}}:{{.Column}}: {{.Message}}\\n{{end}}");
        cmd.arg(filename);
        let inner = linters.spawn("actionlint", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        parse_line_standard(filename, "actionlint", line)
    }
}

impl Stream for GithubWorkflowActionlint {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "actionlint",
            &this.filename,
            &mut this.inner,
            Self::parse_line,
            false,
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_standard() {
        let entry = GithubWorkflowActionlint::parse_line(
            Path::new(".github/workflows/ci.yml"),
            ".github/workflows/ci.yml:8:9: step must run script with \"run\" section or run action with \"uses\" section",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            ".github/workflows/ci.yml:8: [actionlint] step must run script with \"run\" section or run action with \"uses\" section"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(GithubWorkflowActionlint::parse_line(Path::new("x.yml"), "").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(
            GithubWorkflowActionlint::parse_line(Path::new("x.yml"), "x.yml:x:y: msg").is_none()
        );
    }
}
