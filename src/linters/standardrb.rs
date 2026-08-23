// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [standardrb](https://github.com/standardrb/standard) Ruby linter wrapper.
//!
//! standard is a Ruby coding style enforcer built on top of rubocop, using
//! the Standard style guide defaults. It is run with `--format=clang` to
//! produce machine-readable output that is parsed into [`Entry`] values.
//!
//! ## Output format
//!
//! Each line emitted by standardrb on stdout has the form:
//!
//! ```text
//! <filename>:<line>:<col>: [Category] message
//! ```
//!
//! For example:
//!
//! ```text
//! foo.rb:1:1: [Convention] Missing magic comment.
//! ```

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct RubyStandardrb {
    filename: PathBuf,
    inner: Linter,
}

impl RubyStandardrb {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("standardrb");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("--format=clang");
        cmd.arg("--force-exclusion");
        cmd.arg(filename);
        let inner = linters.spawn("standardrb", cmd)?;
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
        let parts: Vec<&str> = line.splitn(4, ':').collect();
        if parts.len() < 4 {
            return None;
        }
        let line_num: u32 = parts[1].trim().parse().ok()?;
        let col_num: u32 = parts[2].trim().parse().ok()?;
        let raw_msg = parts[3].trim();
        // Strip the [Category] prefix
        let msg = raw_msg
            .strip_prefix('[')
            .and_then(|s| s.find(']').map(|i| s[i + 1..].trim()))
            .unwrap_or(raw_msg);
        Some(Entry::new_line_col(filename, "standardrb", msg, line_num, col_num).unwrap())
    }
}

impl Stream for RubyStandardrb {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "standardrb",
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
        let entry = RubyStandardrb::parse_line(
            Path::new("test.rb"),
            "test.rb:1:1: [Convention] Missing magic comment.",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.rb:1: [standardrb] Missing magic comment."
        );
    }

    #[test]
    fn parse_line_with_severity_prefix() {
        let entry = RubyStandardrb::parse_line(
            Path::new("test.rb"),
            "test.rb:3:5: C: [Correctable] Style/StringLiterals: Prefer double-quoted strings",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.rb:3: [standardrb] C: [Correctable] Style/StringLiterals: Prefer double-quoted strings"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(RubyStandardrb::parse_line(Path::new("test.rb"), "").is_none());
    }

    #[test]
    fn parse_line_too_few_parts() {
        assert!(RubyStandardrb::parse_line(Path::new("test.rb"), "no colons here").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(
            RubyStandardrb::parse_line(Path::new("test.rb"), "test.rb:x:y: [Style] msg").is_none()
        );
    }
}
