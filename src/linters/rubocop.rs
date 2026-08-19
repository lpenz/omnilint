// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [rubocop](https://docs.rubocop.org/) Ruby linter wrapper.
//!
//! rubocop checks Ruby source files for coding style and potential errors.
//! It is run with `--format=clang` to produce machine-readable output that
//! is parsed into [`Entry`] values.
//!
//! ## Output format
//!
//! Each line emitted by rubocop on stdout has the form:
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

pub struct RubyRubocop {
    filename: PathBuf,
    inner: Linter,
}

impl RubyRubocop {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("rubocop");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("--format=clang");
        cmd.arg("--force-exclusion");
        cmd.arg(filename);
        let inner = linters.spawn("rubocop", cmd)?;
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
        Some(Entry::new_line_col(filename, "rubocop", msg, line_num, col_num).unwrap())
    }
}

impl Stream for RubyRubocop {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "rubocop",
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
        let entry = RubyRubocop::parse_line(
            Path::new("test.rb"),
            "test.rb:1:1: [Convention] Missing magic comment.",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.rb:1: [rubocop] Missing magic comment."
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(RubyRubocop::parse_line(Path::new("test.rb"), "").is_none());
    }

    #[test]
    fn parse_line_too_few_parts() {
        assert!(RubyRubocop::parse_line(Path::new("test.rb"), "no colons here").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(
            RubyRubocop::parse_line(Path::new("test.rb"), "test.rb:x:y: [Style] msg").is_none()
        );
    }
}
