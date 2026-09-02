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
use crate::linters::{CommandLinter, Linters, Spec, into_entries};

use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio_stream::Stream;

pub struct PythonPylint(CommandLinter);

impl PythonPylint {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        Ok(Self(CommandLinter::new(
            linters,
            Spec {
                name: "pylint",
                args: &["--output-format=text", "--score=no"],
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

linter_stream!(PythonPylint);

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
