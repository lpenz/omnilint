// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [GHC](https://www.haskell.org/ghc/) Haskell syntax checker.
//!
//! `ghc -fno-code` compiles a Haskell file without generating code,
//! checking for syntax and type errors. It serves as a basic fallback
//! alongside hlint.
//!
//! ## Output format
//!
//! Errors are printed to stderr:
//!
//! ```text
//! <filename>:<line>:<col>: <severity>: <message>
//! ```

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct HsGhc {
    filename: PathBuf,
    inner: Linter,
}

impl HsGhc {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("ghc");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("-fno-code");
        cmd.arg(filename);
        let inner = linters.spawn("ghc", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        let line = line.trim();
        let fname = filename.to_str()?;
        let marker = format!("{fname}:");
        let rest = line.strip_prefix(&marker)?;
        let (line_str, rest) = rest.split_once(':')?;
        let line_num: u32 = line_str.parse().ok()?;
        let rest = rest.trim_start();
        let (_, rest) = rest.split_once(':')?;
        let msg = rest.trim();
        Some(Entry::new_line(filename, "ghc", msg, line_num).unwrap())
    }
}

impl Stream for HsGhc {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "ghc",
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
        let entry = HsGhc::parse_line(
            Path::new("Test.hs"),
            "Test.hs:5:1: error: parse error on input ‘x’",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "Test.hs:5: [ghc] error: parse error on input ‘x’"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(HsGhc::parse_line(Path::new("Test.hs"), "").is_none());
    }

    #[test]
    fn parse_line_no_colon() {
        assert!(HsGhc::parse_line(Path::new("Test.hs"), "some warning").is_none());
    }
}
