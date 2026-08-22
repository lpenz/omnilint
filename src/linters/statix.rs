// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [statix](https://github.com/oppiliappan/statix) Nix linter wrapper.
//!
//! statix reports antipatterns in Nix source files. It is run with
//! `check -o errfmt` and its output is parsed into [`Entry`] values.
//!
//! ## Output format
//!
//! Each finding is a single line of the form:
//!
//! ```text
//! foo.nix>2:9:W:12:Consider quoting this URI expression
//! ```

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct NixStatix {
    filename: PathBuf,
    inner: Linter,
}

impl NixStatix {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("statix");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("check");
        cmd.arg("-o");
        cmd.arg("errfmt");
        cmd.arg(filename);
        let inner = linters.spawn("statix", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        let line = line.trim();
        let rest = line.split_once('>')?.1;
        let parts: Vec<&str> = rest.splitn(5, ':').collect();
        if parts.len() < 5 {
            return None;
        }
        let line_num: u32 = parts[0].trim().parse().ok()?;
        let col_num: u32 = parts[1].trim().parse().ok()?;
        // parts[2] is the severity (W), parts[3] is the end column
        let msg = parts[4].trim();
        Some(Entry::new_line_col(filename, "statix", msg, line_num, col_num).unwrap())
    }
}

impl Stream for NixStatix {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "statix",
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
        let entry = NixStatix::parse_line(
            Path::new("test.nix"),
            "test.nix>2:9:W:12:Consider quoting this URI expression",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.nix:2: [statix] Consider quoting this URI expression"
        );
    }

    #[test]
    fn parse_line_absolute_path() {
        let entry = NixStatix::parse_line(
            Path::new("st.nix"),
            "/tmp/st.nix>8:16:W:8:Useless parentheses",
        )
        .unwrap();
        assert_eq!(entry.to_string(), "st.nix:8: [statix] Useless parentheses");
    }

    #[test]
    fn parse_line_no_gt() {
        assert!(NixStatix::parse_line(Path::new("test.nix"), "test.nix:2:9:W:12:msg").is_none());
    }

    #[test]
    fn parse_line_too_few_parts() {
        assert!(NixStatix::parse_line(Path::new("test.nix"), "test.nix>2:9").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(NixStatix::parse_line(Path::new("test.nix"), "test.nix>x:y:W:z:msg").is_none());
    }

    #[test]
    fn parse_line_empty() {
        assert!(NixStatix::parse_line(Path::new("test.nix"), "").is_none());
    }
}
