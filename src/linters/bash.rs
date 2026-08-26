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
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct ShBash {
    filename: PathBuf,
    inner: Linter,
}

impl ShBash {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("bash");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("--norc");
        cmd.arg("-n");
        cmd.arg(filename);
        let inner = linters.spawn("bash", cmd)?;
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

impl Stream for ShBash {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "bash",
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
