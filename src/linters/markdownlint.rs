// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [markdownlint-cli2](https://github.com/DavidAnson/markdownlint-cli2) Markdown linter wrapper.
//!
//! markdownlint-cli2 checks Markdown files for style and correctness. It is
//! run once per file, analysing a single file and creating no build artifacts.
//!
//! ## Output format
//!
//! Each finding emitted by markdownlint-cli2 on stderr has the form:
//!
//! ```text
//! <filename>:<line>[:<col>] <severity> <code> <message>
//! ```
//!
//! For example:
//!
//! ```text
//! markdown-dirty.md:5:32 error MD009/no-trailing-spaces Trailing spaces [Expected: 0 or 2; Actual: 3]
//! ```
//!
//! The column is only present when the rule has a specific location; the
//! severity and rule code are discarded, keeping the message.

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct MarkdownMarkdownlint {
    filename: PathBuf,
    inner: Linter,
}

impl MarkdownMarkdownlint {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("markdownlint-cli2");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg(filename);
        let inner = linters.spawn("markdownlint-cli2", cmd)?;
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
        let (loc, rest) = line.split_once(' ')?;
        let (_, rest) = rest.split_once(' ')?;
        let (_, msg) = rest.split_once(' ')?;
        let mut parts = loc.split(':');
        let _file = parts.next()?;
        let line_num: u32 = parts.next()?.parse().ok()?;
        let col_num: Option<u32> = match parts.next() {
            Some(col) => Some(col.parse().ok()?),
            None => None,
        };
        if parts.next().is_some() {
            return None;
        }
        match col_num {
            Some(col) => Some(
                Entry::new_line_col(filename, "markdownlint-cli2", msg, line_num, col).unwrap(),
            ),
            None => Some(Entry::new_line(filename, "markdownlint-cli2", msg, line_num).unwrap()),
        }
    }
}

impl Stream for MarkdownMarkdownlint {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "markdownlint-cli2",
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
    fn parse_line_with_col() {
        let entry = MarkdownMarkdownlint::parse_line(
            Path::new("foo.md"),
            "foo.md:5:32 error MD009/no-trailing-spaces Trailing spaces [Expected: 0 or 2; Actual: 3]",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "foo.md:5: [markdownlint-cli2] Trailing spaces [Expected: 0 or 2; Actual: 3]"
        );
    }

    #[test]
    fn parse_line_without_col() {
        let entry = MarkdownMarkdownlint::parse_line(
            Path::new("foo.md"),
            "foo.md:4 error MD032/blanks-around-lists Lists should be surrounded by blank lines [Context: \"- item one\"]",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "foo.md:4: [markdownlint-cli2] Lists should be surrounded by blank lines [Context: \"- item one\"]"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(MarkdownMarkdownlint::parse_line(Path::new("foo.md"), "").is_none());
    }

    #[test]
    fn parse_line_unparseable() {
        assert!(MarkdownMarkdownlint::parse_line(Path::new("foo.md"), "garbage").is_none());
    }
}
