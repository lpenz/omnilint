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
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct TeXChktex {
    filename: PathBuf,
    inner: Linter,
}

impl TeXChktex {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("chktex");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg(filename);
        let inner = linters.spawn("chktex", cmd)?;
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
        // Format: "Warning <num> in <filename> line <line>: <message>"
        let line_start = line.find(" line ")?;
        let after_line = &line[line_start + 6..];
        let colon_pos = after_line.find(':')?;
        let line_num: u32 = after_line[..colon_pos].trim().parse().ok()?;
        let msg = after_line[colon_pos + 1..].trim();
        Some(Entry::new_line(filename, "chktex", msg, line_num).unwrap())
    }
}

impl Stream for TeXChktex {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "chktex",
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
