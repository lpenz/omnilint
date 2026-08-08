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

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll, ready};

use color_eyre::Result;
use tokio::process::Command;
use tokio_process_stream::{Item as ProcessItem, ProcessLineStream};
use tokio_stream::Stream;

pub struct PythonRuff {
    filename: PathBuf,
    inner: ProcessLineStream,
}

impl PythonRuff {
    pub fn new(filename: &Path) -> Result<Self> {
        let mut cmd = Command::new("ruff");
        cmd.arg("check");
        cmd.arg("--output-format");
        cmd.arg("concise");
        cmd.arg(filename);
        let inner = ProcessLineStream::try_from(cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        let line = line.trim();
        let parts: Vec<&str> = line.splitn(4, ':').collect();
        if parts.len() < 4 {
            // Not a finding line (e.g. the "Found N errors." summary).
            return None;
        }
        let line_num: u32 = parts[1].parse().ok()?;
        let col_num: u32 = parts[2].parse().ok()?;
        let msg = parts[3].trim();
        if line_num == 0 {
            return Some(Entry::new(filename, "ruff", msg).unwrap());
        }
        Some(Entry::new_line_col(filename, "ruff", msg, line_num, col_num).unwrap())
    }
}

impl Stream for PythonRuff {
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
                    eprintln!("[ruff {}] stderr {}", this.filename.display(), line);
                }
                Some(ProcessItem::Done(_)) => {
                    // ruff ends in error if it finds a violation, we can just ignore it.
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
