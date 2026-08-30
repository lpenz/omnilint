// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [mypy](https://mypy-lang.org/) Python linter wrapper.
//!
//! mypy performs static type checking of Python source files. It is run
//! with `--no-error-summary` and a cache directory below the system
//! temporary directory, and its output is parsed into [`Entry`] values.
//!
//! ## Output format
//!
//! Each finding is emitted as a single line of the form:
//!
//! ```text
//! <filename>:<line>: <severity>: <message>  [<code>]
//! ```
//!
//! For example:
//!
//! ```text
//! foo.py:8: error: Name "x" is not defined  [name-defined]
//! ```
//!
//! The error code between brackets is appended to the message between
//! parentheses, `note` lines are skipped, and lines that don't match the
//! format (e.g. the summary, disabled by `--no-error-summary`) are ignored.

use crate::entry::Entry;
use crate::linters::Linter;
use crate::linters::Linters;

use std::collections::VecDeque;
use std::path::Path;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use color_eyre::Result;
use tokio::process::Command;
use tokio_process_stream::Item as ProcessItem;
use tokio_stream::Stream;

pub struct PythonMypy {
    filename: PathBuf,
    inner: Linter,
    pending: VecDeque<Entry>,
}

impl PythonMypy {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("mypy");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("--no-error-summary");
        let cache_dir = std::env::temp_dir().join("omnilint-mypy-cache");
        let _ = std::fs::create_dir_all(&cache_dir);
        cmd.arg(format!("--cache-dir={}", cache_dir.display()));
        cmd.arg(filename);
        let inner = linters.spawn("mypy", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
            pending: VecDeque::new(),
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Vec<Entry> {
        let line = line.trim();
        if line.is_empty() {
            return Vec::new();
        }
        let parts: Vec<&str> = line.splitn(3, ':').collect();
        if parts.len() < 3 {
            return Vec::new();
        }
        let line_num: u32 = match parts[1].trim().parse() {
            Ok(line_num) => line_num,
            Err(_) => return Vec::new(),
        };
        let rest = parts[2].trim();
        if !rest.starts_with("error: ") && !rest.starts_with("warning: ") {
            // Includes "note" lines, which only add context to other errors.
            return Vec::new();
        }
        let Some((_, rest_msg)) = rest.split_once(": ") else {
            return Vec::new();
        };
        let mut msg = rest_msg.to_string();
        // Remove and append the error code, e.g. "  [name-defined]".
        if msg.ends_with(']')
            && let Some(idx) = msg.rfind("  [")
        {
            let code = msg[idx + 3..msg.len() - 1].to_string();
            msg.truncate(idx);
            msg = format!("{} ({code})", msg.trim_end());
        }
        let entry = Entry::new_line(filename, "mypy", &msg, line_num);
        match entry {
            Ok(entry) => vec![entry],
            Err(_) => Vec::new(),
        }
    }
}

impl Stream for PythonMypy {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Entry>> {
        let this = self.get_mut();
        loop {
            if let Some(entry) = this.pending.pop_front() {
                return Poll::Ready(Some(entry));
            }
            match &mut this.inner {
                Linter::Running(stream) => match Pin::new(&mut **stream).poll_next(cx) {
                    Poll::Ready(Some(ProcessItem::Stdout(line))) => {
                        this.pending.extend(Self::parse_line(&this.filename, &line));
                    }
                    Poll::Ready(Some(_)) => continue,
                    Poll::Ready(None) => return Poll::Ready(None),
                    Poll::Pending => return Poll::Pending,
                },
                Linter::NotFound => {
                    this.inner = Linter::Done;
                    return Poll::Ready(Some(
                        Entry::new(&this.filename, "mypy", "linter not found").unwrap(),
                    ));
                }
                Linter::Done => return Poll::Ready(None),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_standard() {
        let entries = PythonMypy::parse_line(
            Path::new("test.py"),
            r#"test.py:8: error: Name "x" is not defined  [name-defined]"#,
        );
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].to_string(),
            "test.py:8: [mypy] Name \"x\" is not defined (name-defined)"
        );
    }

    #[test]
    fn parse_line_warning() {
        let entries = PythonMypy::parse_line(
            Path::new("test.py"),
            "test.py:2: warning: Something is suspicious",
        );
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].to_string(),
            "test.py:2: [mypy] Something is suspicious"
        );
    }

    #[test]
    fn parse_line_skips_notes() {
        assert!(
            PythonMypy::parse_line(Path::new("test.py"), "test.py:1: note: some hint").is_empty()
        );
    }

    #[test]
    fn parse_line_skips_summary() {
        assert!(
            PythonMypy::parse_line(Path::new("test.py"), "Found 2 errors in 1 file").is_empty()
        );
    }

    #[test]
    fn parse_line_without_code() {
        let entries =
            PythonMypy::parse_line(Path::new("test.py"), "test.py:3: error: plain message");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].to_string(), "test.py:3: [mypy] plain message");
    }

    #[test]
    fn parse_line_empty() {
        assert!(PythonMypy::parse_line(Path::new("test.py"), "").is_empty());
    }

    #[test]
    fn parse_line_too_few_parts() {
        assert!(PythonMypy::parse_line(Path::new("test.py"), "no colons here").is_empty());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(PythonMypy::parse_line(Path::new("test.py"), "test.py:x: error: msg").is_empty());
    }
}
