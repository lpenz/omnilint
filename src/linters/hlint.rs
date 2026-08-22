// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [hlint](https://github.com/ndmitchell/hlint) Haskell linter wrapper.
//!
//! hlint suggests improvements to Haskell source files. It is run with
//! `--no-summary`, and its output is parsed into [`Entry`] values.
//!
//! ## Output format
//!
//! Each hint starts with a single line containing the location, severity
//! and title, followed by multi-line `Found:` / `Perhaps:` blocks that are
//! skipped:
//!
//! ```text
//! foo.hs:7:1-23: Warning: Eta reduce
//! Found:
//!   h xs = foldr (++) [] xs
//! Perhaps:
//!   h = foldr (++) []
//! ```
//!
//! The location may also be a range in the `(line,col)-(line,col)` form.
//! Only the start of the range is used, and the severity is kept as a
//! prefix of the message.

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct HsHlint {
    filename: PathBuf,
    inner: Linter,
}

impl HsHlint {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("hlint");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("--no-summary");
        cmd.arg(filename);
        let inner = linters.spawn("hlint", cmd)?;
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
        for severity in ["Suggestion: ", "Warning: ", "Error: "] {
            if let Some(idx) = line.find(severity) {
                let location = &line[..idx];
                let msg = &line[idx + severity.len()..];
                let msg = format!("{severity}{}", msg.trim_end());
                return Self::parse_location(filename, location, &msg);
            }
        }
        None
    }

    /// Parses the location prefix of a hint line, which is either
    /// `<line>:<col>`, `<line>:<col>-<col>` or `(<line>,<col>)-(<line>,<col>)`.
    fn parse_location(filename: &Path, location: &str, msg: &str) -> Option<Entry> {
        let location = location.trim().trim_end_matches(':');
        let (line_num, col_num) = if let Some(idx) = location.find('(') {
            let inner = location[idx + 1..].split(')').next()?;
            inner.split_once(',')?
        } else {
            let mut parts: Vec<&str> = location.split(':').collect();
            let col_raw = parts.pop()?;
            let line_raw = parts.pop()?;
            (line_raw, col_raw.split('-').next()?)
        };
        let line_num: u32 = line_num.trim().parse().ok()?;
        let col_num: u32 = col_num.trim().parse().ok()?;
        Some(Entry::new_line_col(filename, "hlint", msg, line_num, col_num).unwrap())
    }
}

impl Stream for HsHlint {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "hlint",
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
    fn parse_line_line_col_range() {
        let entry =
            HsHlint::parse_line(Path::new("test.hs"), "test.hs:7:1-23: Warning: Eta reduce")
                .unwrap();
        assert_eq!(entry.to_string(), "test.hs:7: [hlint] Warning: Eta reduce");
    }

    #[test]
    fn parse_line_parenthesized_range() {
        let entry = HsHlint::parse_line(
            Path::new("test.hs"),
            "test.hs:(6,7)-(8,14): Suggestion: Replace case with fromMaybe",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.hs:6: [hlint] Suggestion: Replace case with fromMaybe"
        );
    }

    #[test]
    fn parse_line_single_position() {
        let entry =
            HsHlint::parse_line(Path::new("test.hs"), "test.hs:2:8: Error: Parse error").unwrap();
        assert_eq!(entry.to_string(), "test.hs:2: [hlint] Error: Parse error");
    }

    #[test]
    fn parse_line_skips_continuation_lines() {
        assert!(HsHlint::parse_line(Path::new("test.hs"), "Found:").is_none());
        assert!(HsHlint::parse_line(Path::new("test.hs"), "Perhaps:").is_none());
        assert!(HsHlint::parse_line(Path::new("test.hs"), "  h = foldr (++) []").is_none());
    }

    #[test]
    fn parse_line_empty() {
        assert!(HsHlint::parse_line(Path::new("test.hs"), "").is_none());
    }
}
