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
use crate::linters::{CommandLinter, Linters, Spec, into_entries};

use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio_stream::Stream;

pub struct MarkdownMarkdownlint(CommandLinter);

impl MarkdownMarkdownlint {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        Ok(Self(CommandLinter::new(
            linters,
            Spec {
                name: "markdownlint-cli2",
                findings_on_stderr: true,
                parse: |f, l| into_entries(f, l, Self::parse_line),
                ..Default::default()
            },
            filename,
        )?))
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

linter_stream!(MarkdownMarkdownlint);

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
