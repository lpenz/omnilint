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
use crate::linters::{CommandLinter, Linters, Spec, parse_line_standard};

use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio_stream::Stream;

pub struct PythonRuff(CommandLinter);

impl PythonRuff {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        Ok(Self(CommandLinter::new(
            linters,
            Spec {
                name: "ruff",
                args: &["check", "--output-format", "concise"],
                parse: parse_line,
                ..Default::default()
            },
            filename,
        )?))
    }
}

fn parse_line(filename: &Path, line: &str) -> Vec<Entry> {
    parse_line_standard(filename, "ruff", line)
}

linter_stream!(PythonRuff);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_standard() {
        let entries = parse_line(
            Path::new("test.py"),
            "test.py:1:8: F401 'os' imported but unused",
        );
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].to_string(),
            "test.py:1: [ruff] F401 'os' imported but unused"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(parse_line(Path::new("test.py"), "").is_empty());
    }

    #[test]
    fn parse_line_summary() {
        assert!(parse_line(Path::new("test.py"), "Found 3 errors.").is_empty());
        assert!(
            parse_line(
                Path::new("test.py"),
                "[*] 1 fixable with the `--fix` option"
            )
            .is_empty()
        );
    }

    #[test]
    fn parse_line_zero_line() {
        let entries = parse_line(Path::new("test.py"), "test.py:0:1: some message");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].to_string(), "test.py: [ruff] some message");
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(parse_line(Path::new("test.py"), "test.py:x:y: msg").is_empty());
    }
}
