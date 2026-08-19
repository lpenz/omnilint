// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [pylint](https://pylint.readthedocs.io/) Python linter wrapper.
//!
//! pylint checks Python source files for programming errors and
//! enforces coding standards. It is run with `--output-format=text` to
//! produce machine-readable output that is parsed into [`Entry`] values.
//!
//! ## Output format
//!
//! Each line emitted by pylint on stdout has the form:
//!
//! ```text
//! <filename>:<line>:<col>: <CODE>: <message>
//! ```
//!
//! For example:
//!
//! ```text
//! foo.py:1:0: C0114: Missing module docstring
//! ```

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct PythonPylint {
    filename: PathBuf,
    inner: Linter,
}

impl PythonPylint {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("pylint");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("--output-format=text");
        cmd.arg("--score=no");
        cmd.arg(filename);
        let inner = linters.spawn("pylint", cmd)?;
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
        let parts: Vec<&str> = line.splitn(5, ':').collect();
        if parts.len() < 5 {
            return None;
        }
        let line_num: u32 = parts[1].trim().parse().ok()?;
        let col_num: u32 = parts[2].trim().parse().ok()?;
        // parts[3] is the CODE, parts[4] is " message"
        let msg = parts[4].trim();
        if col_num == 0 {
            Some(Entry::new_line(filename, "pylint", msg, line_num).unwrap())
        } else {
            Some(Entry::new_line_col(filename, "pylint", msg, line_num, col_num).unwrap())
        }
    }
}

impl Stream for PythonPylint {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "pylint",
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
        let entry = PythonPylint::parse_line(
            Path::new("test.py"),
            "test.py:1:0: C0114: Missing module docstring",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.py:1: [pylint] Missing module docstring"
        );
    }

    #[test]
    fn parse_line_with_column() {
        let entry = PythonPylint::parse_line(
            Path::new("test.py"),
            "test.py:1:5: C0114: Missing module docstring",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.py:1: [pylint] Missing module docstring"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(PythonPylint::parse_line(Path::new("test.py"), "").is_none());
    }

    #[test]
    fn parse_line_too_few_parts() {
        assert!(PythonPylint::parse_line(Path::new("test.py"), "no colons here").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(
            PythonPylint::parse_line(Path::new("test.py"), "test.py:x:y: C0114: msg").is_none()
        );
    }
}
