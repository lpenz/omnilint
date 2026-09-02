// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [chktex](https://www.nongnu.org/chktex/) TeX/LaTeX linter wrapper.
//!
//! chktex checks TeX and LaTeX source files for style errors.
//!
//! ## Output format
//!
//! Each line emitted by chktex on stdout has the form:
//!
//! ```text
//! Warning <num> in <filename> line <line>: <message>
//! ```
//!
//! For example:
//!
//! ```text
//! Warning 33 in foo.tex line 1: Use ' to end quotation, not `.
//! ```

use crate::entry::Entry;
use crate::linters::{CommandLinter, Linters, Spec, into_entries};

use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio_stream::Stream;

pub struct TeXChktex(CommandLinter);

impl TeXChktex {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        Ok(Self(CommandLinter::new(
            linters,
            Spec {
                name: "chktex",
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
        // Format: "Warning <num> in <filename> line <line>: <message>"
        let line_start = line.find(" line ")?;
        let after_line = &line[line_start + 6..];
        let colon_pos = after_line.find(':')?;
        let line_num: u32 = after_line[..colon_pos].trim().parse().ok()?;
        let msg = after_line[colon_pos + 1..].trim();
        Some(Entry::new_line(filename, "chktex", msg, line_num).unwrap())
    }
}

linter_stream!(TeXChktex);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_standard() {
        let entry = TeXChktex::parse_line(
            Path::new("test.tex"),
            "Warning 33 in test.tex line 1: Use ' to end quotation, not `.",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.tex:1: [chktex] Use ' to end quotation, not `."
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(TeXChktex::parse_line(Path::new("test.tex"), "").is_none());
    }

    #[test]
    fn parse_line_no_line_keyword() {
        assert!(TeXChktex::parse_line(Path::new("test.tex"), "no line keyword here").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(
            TeXChktex::parse_line(Path::new("test.tex"), "Warning 33 in test.tex line x: msg")
                .is_none()
        );
    }
}
