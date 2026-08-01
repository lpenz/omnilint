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

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll, ready};

use color_eyre::Result;
use tokio::process::Command;
use tokio_process_stream::{Item as ProcessItem, ProcessLineStream};
use tokio_stream::Stream;

pub struct PythonFlake8 {
    filename: PathBuf,
    inner: ProcessLineStream,
}

impl PythonFlake8 {
    pub fn new(filename: &Path) -> Result<Self> {
        let mut cmd = Command::new("flake8");
        cmd.arg(filename);
        let inner = ProcessLineStream::try_from(cmd)?;
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
        let parts: Vec<&str> = line.splitn(4, ':').collect();
        assert!(parts.len() >= 4, "unexpected flake8 output: {line}");
        let line_num: u32 = parts[1].parse().ok()?;
        let col_num: u32 = parts[2].parse().ok()?;
        let msg = parts[3].trim();
        if line_num == 0 {
            return Some(Entry::new(filename, msg).unwrap());
        }
        Some(Entry::new_line_col(filename, msg, line_num, col_num).unwrap())
    }
}

impl Stream for PythonFlake8 {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        loop {
            match ready!(Pin::new(&mut this.inner).poll_next(cx)) {
                Some(ProcessItem::Stdout(line)) => {
                    if let Some(entry) = Self::parse_line(&this.filename, &line) {
                        return Poll::Ready(Some(entry));
                    }
                }
                Some(ProcessItem::Stderr(line)) => {
                    eprintln!("[flake8 {}] stderr {}", this.filename.display(), line);
                }
                Some(ProcessItem::Done(_)) => {
                    // flake8 ends in error if it finds a warning, we can just ignore it.
                    continue;
                }
                None => return Poll::Ready(None),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_standard() {
        let entry = PythonFlake8::parse_line(
            Path::new("test.py"),
            "test.py:1:1: F401 'os' imported but unused",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.py:1:1: F401 'os' imported but unused"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(PythonFlake8::parse_line(Path::new("test.py"), "").is_none());
    }

    #[test]
    fn parse_line_zero_line() {
        let entry =
            PythonFlake8::parse_line(Path::new("test.py"), "test.py:0:1: some message").unwrap();
        assert_eq!(entry.to_string(), "test.py: some message");
    }

    #[test]
    #[should_panic(expected = "unexpected flake8 output")]
    fn parse_line_too_few_parts() {
        PythonFlake8::parse_line(Path::new("test.py"), "no colons here");
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(PythonFlake8::parse_line(Path::new("test.py"), "test.py:x:y: msg").is_none());
    }
}
