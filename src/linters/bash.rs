// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [bash](https://www.gnu.org/software/bash/) shell syntax checker.
//!
//! `bash -n` performs a syntax check on a shell script without executing it.
//! It serves as a fallback alongside shellcheck, catching syntax errors that
//! shellcheck may not report.
//!
//! ## Output format
//!
//! Each finding emitted by bash on stderr has the form:
//!
//! ```text
//! <filename>: <message>
//! ```
//!
//! or with a line number:
//!
//! ```text
//! <filename>:line: <message>
//! ```

use crate::entry::Entry;
use crate::linters::{CommandLinter, Linters, Spec, into_entries};

use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio_stream::Stream;

pub struct ShBash(CommandLinter);

impl ShBash {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        Ok(Self(CommandLinter::new(
            linters,
            Spec {
                name: "bash",
                args: &["--norc", "-n"],
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
        let fname = filename.to_str()?;
        let rest = line.strip_prefix(fname)?;
        let rest = rest.strip_prefix(':')?;
        if let Some(rest) = rest.strip_prefix(' ') {
            return Some(Entry::new(filename, "bash", rest).unwrap());
        }
        let (line_num_str, msg) = rest.split_once(": ")?;
        let line_num: u32 = line_num_str.parse().ok()?;
        Some(Entry::new_line(filename, "bash", msg.trim(), line_num).unwrap())
    }
}

linter_stream!(ShBash);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_with_line_number() {
        let entry = ShBash::parse_line(
            Path::new("test.sh"),
            "test.sh:3: syntax error near unexpected token `fi'",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.sh:3: [bash] syntax error near unexpected token `fi'"
        );
    }

    #[test]
    fn parse_line_without_line_number() {
        let entry =
            ShBash::parse_line(Path::new("test.sh"), "test.sh: syntax error in file").unwrap();
        assert_eq!(entry.to_string(), "test.sh: [bash] syntax error in file");
    }

    #[test]
    fn parse_line_empty() {
        assert!(ShBash::parse_line(Path::new("test.sh"), "").is_none());
    }

    #[test]
    fn parse_line_wrong_prefix() {
        assert!(ShBash::parse_line(Path::new("test.sh"), "other.sh:1: msg").is_none());
    }
}
