// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [lintr](https://lintr.r-lib.org/) R linter wrapper.
//!
//! lintr checks R source files for style, syntax and possible semantic
//! issues. It is run through `Rscript` with `--vanilla` so that user and
//! site profiles do not interfere, and its output is parsed into [`Entry`]
//! values.
//!
//! `GITHUB_ACTIONS` is removed from the environment of the spawned process:
//! when it is set to `true` (e.g. when omnilint itself runs on a GitHub
//! Actions runner), lintr's `print.lints()` emits GitHub annotations such as
//! `::warning file=...,line=...` instead of the standard format parsed below.
//!
//! ## Output format
//!
//! Each lint is a single line emitted on stdout, with the form:
//!
//! ```text
//! <filename>:<line>:<col>: <type>: [<linter name>] <message>
//! ```
//!
//! For example:
//!
//! ```text
//! foo.R:1:3: style: [assignment_linter] Use one of <-, <<- for assignment, not =.
//! ```

use crate::entry::Entry;
use crate::linters::{Linter, Linters, parse_line_standard};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct RLintr {
    filename: PathBuf,
    inner: Linter,
}

impl RLintr {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("Rscript");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("--vanilla");
        cmd.arg("-e");
        cmd.arg("lintr::lint(commandArgs(TRUE)[1])");
        cmd.arg(filename);
        cmd.env_remove("GITHUB_ACTIONS");
        let inner = linters.spawn("lintr", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        parse_line_standard(filename, "lintr", line)
    }
}

impl Stream for RLintr {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "lintr",
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
        let entry = RLintr::parse_line(
            Path::new("test.R"),
            "test.R:1:3: style: [assignment_linter] Use one of <-, <<- for assignment, not =.",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.R:1: [lintr] style: [assignment_linter] Use one of <-, <<- for assignment, not =."
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(RLintr::parse_line(Path::new("test.R"), "").is_none());
    }

    #[test]
    fn parse_line_no_lints_found() {
        assert!(RLintr::parse_line(Path::new("test.R"), "No lints found").is_none());
    }
}
