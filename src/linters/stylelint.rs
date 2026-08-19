// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [stylelint](https://stylelint.io/) CSS/SCSS/Less linter wrapper.
//!
//! stylelint checks CSS, SCSS, and Less source files for style errors.
//! It is run with `--formatter=unix` to produce machine-readable output
//! that is parsed into [`Entry`] values.
//!
//! ## Output format
//!
//! Each line emitted by stylelint on stdout has the form:
//!
//! ```text
//! <filename>:<line>:<col>: <message>
//! ```
//!
//! For example:
//!
//! ```text
//! foo.css:1:1: Unexpected empty block (block-no-empty)
//! ```

use crate::entry::Entry;
use crate::linters::{Linter, Linters, parse_line_standard};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct CssStylelint {
    filename: PathBuf,
    inner: Linter,
}

impl CssStylelint {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("stylelint");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("--formatter=unix");
        cmd.arg(filename);
        let inner = linters.spawn("stylelint", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        parse_line_standard(filename, "stylelint", line)
    }
}

impl Stream for CssStylelint {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "stylelint",
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
    fn parse_line_standard() {
        let entry = CssStylelint::parse_line(
            Path::new("test.css"),
            "test.css:1:1: Unexpected empty block (block-no-empty)",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.css:1: [stylelint] Unexpected empty block (block-no-empty)"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(CssStylelint::parse_line(Path::new("test.css"), "").is_none());
    }

    #[test]
    fn parse_line_too_few_parts() {
        assert!(CssStylelint::parse_line(Path::new("test.css"), "no colons here").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(CssStylelint::parse_line(Path::new("test.css"), "test.css:x:y: msg").is_none());
    }
}
