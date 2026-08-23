// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [buildifier](https://github.com/bazelbuild/buildtools) Bazel linter
//! wrapper.
//!
//! buildifier checks Bazel `BUILD`, `WORKSPACE` and `.bzl` Starlark files
//! for style and best practice violations. It is run with `--lint=warn`,
//! and its output is parsed into [`Entry`] values.
//!
//! ## Output format
//!
//! Each warning starts with a single line on stderr, optionally followed by
//! continuation lines that are skipped. A reference URL may be appended to
//! the message, either inline or as a separate line; trailing inline URLs
//! are stripped:
//!
//! ```text
//! <filename>:<line>: <check name>: <message>
//! ```
//!
//! For example:
//!
//! ```text
//! BUILD:3: string literals should use double quotes. [BuildifierWarning] [WC-0004]
//! ```

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct BazelBuildifier {
    filename: PathBuf,
    inner: Linter,
}

impl BazelBuildifier {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("buildifier");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("--lint=warn");
        cmd.arg(filename);
        let inner = linters.spawn("buildifier", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        let line = line.trim();
        let parts: Vec<&str> = line.splitn(3, ':').collect();
        if parts.len() < 3 {
            return None;
        }
        let num: u32 = parts[1].trim().parse().ok()?;
        let msg = parts[2].trim();
        // Strip a trailing reference URL, which buildifier may append to the
        // message line or print on a continuation line
        let msg = match (msg.rfind(" (http"), msg.ends_with(')')) {
            (Some(idx), true) => &msg[..idx],
            _ => msg,
        };
        if msg.is_empty() {
            return None;
        }
        Some(Entry::new_line(filename, "buildifier", msg, num).unwrap())
    }
}

impl Stream for BazelBuildifier {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "buildifier",
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
        let entry = BazelBuildifier::parse_line(
            Path::new("BUILD"),
            "BUILD:3: string literals should use double quotes. [BuildifierWarning] [WC-0004]",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "BUILD:3: [buildifier] string literals should use double quotes. [BuildifierWarning] [WC-0004]"
        );
    }

    #[test]
    fn parse_line_module_docstring() {
        let entry = BazelBuildifier::parse_line(
            Path::new("test.bzl"),
            "/tmp/test.bzl:1: module-docstring: The file has no module docstring.",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.bzl:1: [buildifier] module-docstring: The file has no module docstring."
        );
    }

    #[test]
    fn parse_line_strips_inline_url() {
        let entry = BazelBuildifier::parse_line(
            Path::new("test.bzl"),
            "/tmp/test.bzl:1: unused-variable: Variable \"ctx\" is unused. Please remove it. (https://github.com/bazelbuild/buildtools/blob/main/WARNINGS.md#unused-variable)",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.bzl:1: [buildifier] unused-variable: Variable \"ctx\" is unused. Please remove it."
        );
    }

    #[test]
    fn parse_line_skips_continuation() {
        assert!(
            BazelBuildifier::parse_line(
                Path::new("test.bzl"),
                "A module docstring is a string literal (not a comment)."
            )
            .is_none()
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(BazelBuildifier::parse_line(Path::new("test.bzl"), "").is_none());
    }

    #[test]
    fn parse_line_too_few_parts() {
        assert!(BazelBuildifier::parse_line(Path::new("test.bzl"), "no colons here").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(BazelBuildifier::parse_line(Path::new("test.bzl"), "test.bzl:x: msg").is_none());
    }
}
