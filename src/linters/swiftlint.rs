// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [swiftlint](https://github.com/realm/SwiftLint) Swift linter wrapper.
//!
//! swiftlint checks Swift files for style and common errors. It is run once
//! per file in quiet mode so that only the findings are printed, with
//! SourceKit disabled so that no warnings are emitted about rules that would
//! require it, analysing a single file and creating no build artifacts.
//!
//! ## Output format
//!
//! Each finding emitted by swiftlint on stdout has the form:
//!
//! ```text
//! <filename>:<line>:<col>: <severity>: <message>
//! ```
//!
//! For example:
//!
//! ```text
//! foo.swift:3:5: error: Identifier Name Violation: Variable name 'x' ...
//! ```
//!
//! The `<severity>: ` part between the column and the message is dropped
//! before the message is stored in the [`Entry`].

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct SwiftSwiftlint {
    filename: PathBuf,
    inner: Linter,
}

impl SwiftSwiftlint {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let mut cmd = Command::new("swiftlint");
        cmd.arg("lint");
        cmd.arg("--quiet");
        cmd.arg("--disable-sourcekit");
        cmd.arg(filename);
        let inner = linters.spawn("swiftlint", cmd)?;
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
        let parts: Vec<&str> = line.splitn(5, ':').collect();
        if parts.len() < 5 {
            return None;
        }
        let line_num: u32 = parts.get(1)?.trim().parse().ok()?;
        let col_num: u32 = parts.get(2)?.trim().parse().ok()?;
        let msg = parts.get(4)?.trim();
        Some(Entry::new_line_col(filename, "swiftlint", msg, line_num, col_num).unwrap())
    }
}

impl Stream for SwiftSwiftlint {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "swiftlint",
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
        let entry = SwiftSwiftlint::parse_line(
            Path::new("foo.swift"),
            "foo.swift:3:5: error: Identifier Name Violation: Variable name 'x' should be between 3 and 40 characters long (identifier_name)",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "foo.swift:3: [swiftlint] Identifier Name Violation: Variable name 'x' should be between 3 and 40 characters long (identifier_name)"
        );
    }

    #[test]
    fn parse_line_warning_severity() {
        let entry = SwiftSwiftlint::parse_line(
            Path::new("foo.swift"),
            "foo.swift:1:1: warning: Trailing Whitespace Violation: Line should not have trailing whitespace (trailing_whitespace)",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "foo.swift:1: [swiftlint] Trailing Whitespace Violation: Line should not have trailing whitespace (trailing_whitespace)"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(SwiftSwiftlint::parse_line(Path::new("foo.swift"), "").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(
            SwiftSwiftlint::parse_line(Path::new("foo.swift"), "foo.swift:x:y: error: msg")
                .is_none()
        );
    }
}
