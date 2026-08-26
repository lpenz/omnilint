// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [zsh](https://zsh.sourceforge.io/) shell syntax checker.
//!
//! `zsh --no-exec` performs a syntax check on a zsh script without executing
//! it. This provides coverage for `.zsh` files that shellcheck (which only
//! supports bash/sh) does not cover.
//!
//! ## Output format
//!
//! Each finding emitted by zsh on stderr has the form:
//!
//! ```text
//! <filename>:<line>: <message>
//! ```

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct ShZsh {
    filename: PathBuf,
    inner: Linter,
}

impl ShZsh {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("zsh");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("--no-exec");
        cmd.arg("--no-globalrcs");
        cmd.arg("--no-rcs");
        cmd.arg(filename);
        let inner = linters.spawn("zsh", cmd)?;
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
        let rest = line.strip_prefix(fname)?;
        let rest = rest.strip_prefix(":")?;
        let (line_num_str, msg) = rest.split_once(": ")?;
        let line_num: u32 = line_num_str.parse().ok()?;
        Some(Entry::new_line(filename, "zsh", msg.trim(), line_num).unwrap())
    }
}

impl Stream for ShZsh {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "zsh",
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
        let entry =
            ShZsh::parse_line(Path::new("test.zsh"), "test.zsh:3: parse error near `fi'").unwrap();
        assert_eq!(entry.to_string(), "test.zsh:3: [zsh] parse error near `fi'");
    }

    #[test]
    fn parse_line_empty() {
        assert!(ShZsh::parse_line(Path::new("test.zsh"), "").is_none());
    }

    #[test]
    fn parse_line_wrong_prefix() {
        assert!(ShZsh::parse_line(Path::new("test.zsh"), "other.zsh:1: msg").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(ShZsh::parse_line(Path::new("test.zsh"), "test.zsh:x: msg").is_none());
    }
}
