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
use crate::linters::{CommandLinter, Linters, Spec, into_entries};

use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio_stream::Stream;

pub struct RubyRubocop(CommandLinter);

impl RubyRubocop {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        Ok(Self(CommandLinter::new(
            linters,
            Spec {
                name: "rubocop",
                args: &["--format=clang", "--force-exclusion"],
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

linter_stream!(RubyRubocop);

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
