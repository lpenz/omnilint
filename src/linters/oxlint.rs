// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [oxlint](https://oxc.rs/) JavaScript/TypeScript linter wrapper.
//!
//! oxlint checks JavaScript and TypeScript source files for code quality
//! issues. It is run with `--format=unix` to produce machine-readable
//! output that is parsed into [`Entry`] values.
//!
//! ## Output format
//!
//! Each line emitted by oxlint on stdout has the form:
//!
//! ```text
//! <filename>:<line>:<col>: <message>
//! ```
//!
//! For example:
//!
//! ```text
//! foo.js:1:5: Unexpected var, use let or const instead
//! ```

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct JsOxlint {
    filename: PathBuf,
    inner: Linter,
}

impl JsOxlint {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("oxlint");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("--format=unix");
        cmd.arg(filename);
        let inner = linters.spawn("oxlint", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }
        // unix format: filename:line:col: message
        let parts: Vec<&str> = line.splitn(4, ':').collect();
        if parts.len() < 4 {
            return None;
        }
        let line_num: u32 = parts[1].trim().parse().ok()?;
        let col_num: u32 = parts[2].trim().parse().ok()?;
        let msg = parts[3].trim();
        Some(Entry::new_line_col(filename, "oxlint", msg, line_num, col_num).unwrap())
    }
}

impl Stream for JsOxlint {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "oxlint",
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
        let entry = JsOxlint::parse_line(
            Path::new("test.js"),
            "test.js:1:5: Unexpected var, use let or const instead",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.js:1: [oxlint] Unexpected var, use let or const instead"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(JsOxlint::parse_line(Path::new("test.js"), "").is_none());
    }

    #[test]
    fn parse_line_too_few_parts() {
        assert!(JsOxlint::parse_line(Path::new("test.js"), "no colons here").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(JsOxlint::parse_line(Path::new("test.js"), "test.js:x:y: msg").is_none());
    }
}
