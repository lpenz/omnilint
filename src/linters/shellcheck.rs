// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [shellcheck](https://www.shellcheck.net/) Shell script linter wrapper.
//!
//! ShellCheck checks shell scripts for syntax errors, semantic issues,
//! and common pitfalls. It is run once per file with `-f gcc` to produce
//! machine-readable output that is parsed into [`Entry`] values.
//!
//! ## Output format
//!
//! Each line emitted by shellcheck on stdout has the form:
//!
//! ```text
//! <filename>:<line>:<col>: <severity>: <message> [<code>]
//! ```
//!
//! For example:
//!
//! ```text
//! script.sh:1:1: warning: shebang not present [SC2148]
//! script.sh:12:20: note: Use $((..)) for arithmetics [SC2003]
//! ```
//!
//! The `<severity>: ` prefix and ` [<code>]` suffix are stripped by the
//! parser before the message is stored in the [`Entry`].

use crate::entry::Entry;
use crate::linters::Linter;

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct ShShellcheck {
    filename: PathBuf,
    inner: Linter,
}

impl ShShellcheck {
    pub fn new(filename: &Path) -> Result<Self> {
        let mut cmd = Command::new("shellcheck");
        cmd.arg("-f");
        cmd.arg("gcc");
        cmd.arg(filename);
        let inner = Linter::spawn(cmd)?;
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
        assert!(parts.len() >= 5, "unexpected shellcheck output: {line}");
        let line_num: u32 = parts[1].trim().parse().ok()?;
        let col_num: u32 = parts[2].trim().parse().ok()?;
        let raw_msg = parts[4].trim();
        // Strip the severity prefix (e.g. "warning: ") and the [code] suffix
        let msg = raw_msg
            .strip_prefix('[')
            .and_then(|s| s.find(']').map(|i| s[i + 1..].trim()))
            .unwrap_or(raw_msg);
        let msg = msg.split_once(" [").map_or(msg, |(before, _)| before);
        Some(Entry::new_line_col(filename, "shellcheck", msg, line_num, col_num).unwrap())
    }
}

impl Stream for ShShellcheck {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "shellcheck",
            &this.filename,
            &mut this.inner,
            Self::parse_line,
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_standard() {
        let entry = ShShellcheck::parse_line(
            Path::new("script.sh"),
            "script.sh:1:1: warning: shebang not present [SC2148]",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "script.sh:1: [shellcheck] shebang not present"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(ShShellcheck::parse_line(Path::new("script.sh"), "").is_none());
    }

    #[test]
    #[should_panic(expected = "unexpected shellcheck output")]
    fn parse_line_too_few_parts() {
        ShShellcheck::parse_line(Path::new("script.sh"), "no colons here");
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(
            ShShellcheck::parse_line(
                Path::new("script.sh"),
                "script.sh:x:y: warning: msg [SC9999]"
            )
            .is_none()
        );
    }
}
