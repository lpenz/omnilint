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
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct PythonMypy {
    filename: PathBuf,
    inner: Linter,
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
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }
        let parts: Vec<&str> = line.splitn(3, ':').collect();
        if parts.len() < 3 {
            return None;
        }
        let line_num: u32 = parts[1].trim().parse().ok()?;
        let rest = parts[2].trim();
        if !rest.starts_with("error: ") && !rest.starts_with("warning: ") {
            // Includes "note" lines, which only add context to other errors.
            return None;
        }
        let mut msg = rest.split_once(": ")?.1.to_string();
        // Remove and append the error code, e.g. "  [name-defined]".
        if msg.ends_with(']')
            && let Some(idx) = msg.rfind("  [")
        {
            let code = msg[idx + 3..msg.len() - 1].to_string();
            msg.truncate(idx);
            msg = format!("{} ({code})", msg.trim_end());
        }
        Some(Entry::new_line(filename, "mypy", &msg, line_num).unwrap())
    }
}

impl Stream for PythonMypy {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "mypy",
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
        let entry = PythonMypy::parse_line(
            Path::new("test.py"),
            r#"test.py:8: error: Name "x" is not defined  [name-defined]"#,
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.py:8: [mypy] Name \"x\" is not defined (name-defined)"
        );
    }

    #[test]
    fn parse_line_warning() {
        let entry = PythonMypy::parse_line(
            Path::new("test.py"),
            "test.py:2: warning: Something is suspicious",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.py:2: [mypy] Something is suspicious"
        );
    }

    #[test]
    fn parse_line_skips_notes() {
        assert!(
            PythonMypy::parse_line(Path::new("test.py"), "test.py:1: note: some hint").is_none()
        );
    }

    #[test]
    fn parse_line_skips_summary() {
        assert!(PythonMypy::parse_line(Path::new("test.py"), "Found 2 errors in 1 file").is_none());
    }

    #[test]
    fn parse_line_without_code() {
        let entry = PythonMypy::parse_line(Path::new("test.py"), "test.py:3: error: plain message")
            .unwrap();
        assert_eq!(entry.to_string(), "test.py:3: [mypy] plain message");
    }

    #[test]
    fn parse_line_empty() {
        assert!(PythonMypy::parse_line(Path::new("test.py"), "").is_none());
    }

    #[test]
    fn parse_line_too_few_parts() {
        assert!(PythonMypy::parse_line(Path::new("test.py"), "no colons here").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(PythonMypy::parse_line(Path::new("test.py"), "test.py:x: error: msg").is_none());
    }
}
