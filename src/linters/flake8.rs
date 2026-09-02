// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [flake8](https://flake8.pycqa.org/) Python linter wrapper.
//!
//! flake8 checks Python source files for style errors (PEP 8), programming
//! errors, and complexity issues. It is run once per file and its output is
//! parsed into [`Entry`] values.
//!
//! ## Output format
//!
//! Each line emitted by flake8 on stdout has the form:
//!
//! ```text
//! <filename>:<line>:<col>: <code> <message>
//! ```
//!
//! For example:
//!
//! ```text
//! mymodule.py:12:1: F401 'os' imported but unused
//! mymodule.py:45:80: E501 line too long (95 > 79 characters)
//! ```
//!
//! When the line number is `0`, the issue is file-level (e.g. a syntax
//! error) and no column is reported:
//!
//! ```text
//! mymodule.py:0:0: E999 SyntaxError: invalid syntax
//! ```

use crate::entry::Entry;
use crate::linters::{CommandLinter, Linters, Spec, parse_line_standard};

use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio_stream::Stream;

pub struct PythonFlake8(CommandLinter);

impl PythonFlake8 {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        Ok(Self(CommandLinter::new(
            linters,
            Spec {
                name: "flake8",
                parse: parse_line,
                ..Default::default()
            },
            filename,
        )?))
    }
}

fn parse_line(filename: &Path, line: &str) -> Vec<Entry> {
    parse_line_standard(filename, "flake8", line)
}

linter_stream!(PythonFlake8);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_standard() {
        let entries = parse_line(
            Path::new("test.py"),
            "test.py:1:1: F401 'os' imported but unused",
        );
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].to_string(),
            "test.py:1: [flake8] F401 'os' imported but unused"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(parse_line(Path::new("test.py"), "").is_empty());
    }

    #[test]
    fn parse_line_zero_line() {
        let entries = parse_line(Path::new("test.py"), "test.py:0:1: some message");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].to_string(), "test.py: [flake8] some message");
    }

    #[test]
    fn parse_line_too_few_parts() {
        assert!(parse_line(Path::new("test.py"), "no colons here").is_empty());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(parse_line(Path::new("test.py"), "test.py:x:y: msg").is_empty());
    }
}
