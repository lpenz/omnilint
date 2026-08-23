// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [phpcs](https://github.com/PHPCSStandards/PHP_CodeSniffer) PHP linter
//! wrapper.
//!
//! phpcs checks PHP source files against a coding standard. It is run with
//! `--report=emacs` to produce machine-readable output that is parsed into
//! [`Entry`] values.
//!
//! ## Output format
//!
//! Each finding is a single line emitted on stdout, with the form:
//!
//! ```text
//! <filename>:<line>:<col>: <severity> - <message>
//! ```
//!
//! For example:
//!
//! ```text
//! foo.php:2:13: error - Expected 0 spaces between parenthesis of function declaration; 1 found
//! ```
//!
//! The summary line (`Time: ...`) is skipped.

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct PhpPhpcs {
    filename: PathBuf,
    inner: Linter,
}

impl PhpPhpcs {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("phpcs");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("--report=emacs");
        cmd.arg(filename);
        let inner = linters.spawn("phpcs", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        let line = line.trim();
        let parts: Vec<&str> = line.splitn(4, ':').collect();
        if parts.len() < 4 {
            return None;
        }
        let line_num: u32 = parts[1].trim().parse().ok()?;
        let col_num: u32 = parts[2].trim().parse().ok()?;
        // Strip the "<severity> - " prefix from the message
        let raw_msg = parts[3].trim();
        let msg = raw_msg.split_once(" - ").map(|(_, m)| m.trim());
        Some(
            Entry::new_line_col(filename, "phpcs", msg.unwrap_or(raw_msg), line_num, col_num)
                .unwrap(),
        )
    }
}

impl Stream for PhpPhpcs {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "phpcs",
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
        let entry = PhpPhpcs::parse_line(
            Path::new("test.php"),
            "test.php:2:13: error - Expected 0 spaces between parenthesis of function declaration; 1 found",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.php:2: [phpcs] Expected 0 spaces between parenthesis of function declaration; 1 found"
        );
    }

    #[test]
    fn parse_line_warning() {
        let entry = PhpPhpcs::parse_line(
            Path::new("test.php"),
            "test.php:3:1: warning - Line too long",
        )
        .unwrap();
        assert_eq!(entry.to_string(), "test.php:3: [phpcs] Line too long");
    }

    #[test]
    fn parse_line_skips_time_summary() {
        assert!(PhpPhpcs::parse_line(Path::new("test.php"), "Time: 51ms; Memory: 6MB").is_none());
    }

    #[test]
    fn parse_line_empty() {
        assert!(PhpPhpcs::parse_line(Path::new("test.php"), "").is_none());
    }

    #[test]
    fn parse_line_too_few_parts() {
        assert!(PhpPhpcs::parse_line(Path::new("test.php"), "no colons here").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(PhpPhpcs::parse_line(Path::new("test.php"), "test.php:x:y: error - msg").is_none());
    }
}
