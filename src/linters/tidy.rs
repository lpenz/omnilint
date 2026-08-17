// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [tidy](https://www.html-tidy.org/) HTML linter wrapper.
//!
//! tidy checks HTML files for errors and warnings. It is run once per file
//! with `-errors -quiet` so that only the findings are printed, analysing a
//! single file and creating no build artifacts.
//!
//! ## Output format
//!
//! Each finding emitted by tidy on stderr has the form:
//!
//! ```text
//! line <line> column <col> - Warning: <message>
//! ```
//!
//! For example:
//!
//! ```text
//! line 1 column 1 - Warning: missing <!DOCTYPE> declaration
//! ```
//!
//! The output does not contain the filename, so it is taken from the argument
//! the linter was invoked with.

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct HtmlTidy {
    filename: PathBuf,
    inner: Linter,
}

impl HtmlTidy {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("tidy");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("-errors");
        cmd.arg("-quiet");
        cmd.arg(filename);
        let inner = linters.spawn("tidy", cmd)?;
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
        let rest = line.strip_prefix("line ")?;
        let (line_num, rest) = rest.split_once(" column ")?;
        let (col_num, rest) = rest.split_once(" - ")?;
        let msg = rest
            .strip_prefix("Warning: ")
            .or_else(|| rest.strip_prefix("Error: "))?;
        let line_num: u32 = line_num.parse().ok()?;
        let col_num: u32 = col_num.parse().ok()?;
        Some(Entry::new_line_col(filename, "tidy", msg, line_num, col_num).unwrap())
    }
}

impl Stream for HtmlTidy {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "tidy",
            &this.filename,
            &mut this.inner,
            Self::parse_line,
            true,
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_warning() {
        let entry = HtmlTidy::parse_line(
            Path::new("foo.html"),
            "line 1 column 1 - Warning: missing <!DOCTYPE> declaration",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "foo.html:1: [tidy] missing <!DOCTYPE> declaration"
        );
    }

    #[test]
    fn parse_line_error() {
        let entry = HtmlTidy::parse_line(
            Path::new("foo.html"),
            "line 3 column 5 - Error: <title> element not allowed",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "foo.html:3: [tidy] <title> element not allowed"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(HtmlTidy::parse_line(Path::new("foo.html"), "").is_none());
    }

    #[test]
    fn parse_line_unparseable() {
        assert!(HtmlTidy::parse_line(Path::new("foo.html"), "garbage").is_none());
    }
}
