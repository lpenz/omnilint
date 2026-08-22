// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [pyright](https://microsoft.github.io/pyright/) Python linter wrapper.
//!
//! pyright performs static type checking of Python source files. It is run
//! with its default plain text output format, which is parsed into
//! [`Entry`] values.
//!
//! ## Output format
//!
//! The output contains one indented line per diagnostic, preceded by a
//! line with the file path and followed by a summary line:
//!
//! ```text
//! /abs/path/foo.py
//!   /abs/path/foo.py:8:5 - error: "x" is not defined (reportUndefinedVariable)
//! 1 error, 0 warnings, 0 informations
//! ```
//!
//! Diagnostic lines are parsed into entries with the rule identifier kept
//! between parentheses, and any other line is ignored.

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct PythonPyright {
    filename: PathBuf,
    inner: Linter,
}

impl PythonPyright {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("pyright");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg(filename);
        let inner = linters.spawn("pyright", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        let line = line.trim();
        let (location, rest) = line.split_once(" - ")?;
        // location has the form <path>:<line>:<col>
        let mut location_parts = location.rsplitn(3, ':');
        let col_num: u32 = location_parts.next()?.parse().ok()?;
        let line_num: u32 = location_parts.next()?.parse().ok()?;
        for severity in ["error: ", "warning: ", "information: "] {
            if let Some(msg) = rest.strip_prefix(severity) {
                return Some(
                    Entry::new_line_col(filename, "pyright", msg, line_num, col_num).unwrap(),
                );
            }
        }
        None
    }
}

impl Stream for PythonPyright {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "pyright",
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
        let entry = PythonPyright::parse_line(
            Path::new("test.py"),
            r#"/abs/path/test.py:8:5 - error: "x" is not defined (reportUndefinedVariable)"#,
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.py:8: [pyright] \"x\" is not defined (reportUndefinedVariable)"
        );
    }

    #[test]
    fn parse_line_warning() {
        let entry = PythonPyright::parse_line(
            Path::new("test.py"),
            "/abs/path/test.py:2:1 - warning: something suspicious",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.py:2: [pyright] something suspicious"
        );
    }

    #[test]
    fn parse_line_skips_header_and_summary() {
        assert!(PythonPyright::parse_line(Path::new("test.py"), "/abs/path/test.py").is_none());
        assert!(
            PythonPyright::parse_line(Path::new("test.py"), "1 error, 0 warnings, 0 informations")
                .is_none()
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(PythonPyright::parse_line(Path::new("test.py"), "").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(
            PythonPyright::parse_line(Path::new("test.py"), "/a/b.py:x:y - error: msg").is_none()
        );
    }
}
