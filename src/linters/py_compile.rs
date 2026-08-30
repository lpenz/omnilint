// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [py_compile](https://docs.python.org/3/library/py_compile.html) Python
//! syntax checker.
//!
//! `python3 -m py_compile` compiles a Python file to bytecode, checking for
//! syntax errors. It serves as a basic fallback alongside flake8, ruff,
//! pylint, mypy, and pyright.
//!
//! ## Output format
//!
//! Syntax errors are printed to stderr in the form:
//!
//! ```text
//!   File "<filename>", line <line>
//!     <source line>
//!   <error type>: <message>
//! ```

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct PythonPyCompile {
    filename: PathBuf,
    inner: Linter,
}

impl PythonPyCompile {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable_for_linter("py_compile");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("-m");
        cmd.arg("py_compile");
        cmd.arg(filename);
        let inner = linters.spawn("py_compile", cmd)?;
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
        let fname = filename.to_str()?;
        let marker = format!("File \"{fname}\", line ");
        let rest = line.strip_prefix(&marker)?;
        let line_num: u32 = rest.parse().ok()?;
        Some(Entry::new_line(filename, "py_compile", "syntax error", line_num).unwrap())
    }
}

impl Stream for PythonPyCompile {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "py_compile",
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
        let entry = PythonPyCompile::parse_line(Path::new("test.py"), "  File \"test.py\", line 3")
            .unwrap();
        assert_eq!(entry.to_string(), "test.py:3: [py_compile] syntax error");
    }

    #[test]
    fn parse_line_empty() {
        assert!(PythonPyCompile::parse_line(Path::new("test.py"), "").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(
            PythonPyCompile::parse_line(Path::new("test.py"), "  File \"test.py\", line x")
                .is_none()
        );
    }

    #[test]
    fn parse_line_wrong_prefix() {
        assert!(
            PythonPyCompile::parse_line(Path::new("test.py"), "  File \"other.py\", line 1")
                .is_none()
        );
    }
}
