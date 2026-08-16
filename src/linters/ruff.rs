// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [ruff](https://docs.astral.sh/ruff/) Python linter wrapper.
//!
//! ruff checks Python source files for style errors, programming errors, and
//! complexity issues. It is run once per file with `--output-format concise`
//! and its output is parsed into [`Entry`] values.
//!
//! ## Output format
//!
//! Each finding emitted by ruff on stdout has the form:
//!
//! ```text
//! <filename>:<line>:<col>: <code> <message>
//! ```
//!
//! For example:
//!
//! ```text
//! mymodule.py:1:8: F401 'os' imported but unused
//! mymodule.py:8:5: F821 undefined name 'undefined_name'
//! ```
//!
//! Ruff also prints a summary (e.g. `Found 3 errors.`) and fixability notes to
//! stdout; these lines are not in the finding format and are skipped.

use crate::entry::Entry;
use crate::linters::{Linter, Linters, parse_line_standard};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct PythonRuff {
    filename: PathBuf,
    inner: Linter,
}

impl PythonRuff {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let mut cmd = Command::new("ruff");
        cmd.arg("check");
        cmd.arg("--output-format");
        cmd.arg("concise");
        cmd.arg(filename);
        let inner = linters.spawn("ruff", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        parse_line_standard(filename, "ruff", line)
    }
}

impl Stream for PythonRuff {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "ruff",
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
        let entry = PythonRuff::parse_line(
            Path::new("test.py"),
            "test.py:1:8: F401 'os' imported but unused",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.py:1: [ruff] F401 'os' imported but unused"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(PythonRuff::parse_line(Path::new("test.py"), "").is_none());
    }

    #[test]
    fn parse_line_summary() {
        assert!(PythonRuff::parse_line(Path::new("test.py"), "Found 3 errors.").is_none());
        assert!(
            PythonRuff::parse_line(
                Path::new("test.py"),
                "[*] 1 fixable with the `--fix` option"
            )
            .is_none()
        );
    }

    #[test]
    fn parse_line_zero_line() {
        let entry =
            PythonRuff::parse_line(Path::new("test.py"), "test.py:0:1: some message").unwrap();
        assert_eq!(entry.to_string(), "test.py: [ruff] some message");
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(PythonRuff::parse_line(Path::new("test.py"), "test.py:x:y: msg").is_none());
    }
}
