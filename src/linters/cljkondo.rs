// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [clj-kondo](https://github.com/clj-kondo/clj-kondo) Clojure linter
//! wrapper.
//!
//! clj-kondo lints Clojure, ClojureScript and EDN files for syntax errors,
//! unresolved symbols and common pitfalls. It is run once per file with
//! caching and user configuration disabled so that it analyses a single file
//! without creating any cache artifacts.
//!
//! ## Output format
//!
//! Each finding emitted by clj-kondo on stdout has the form:
//!
//! ```text
//! <filename>:<line>:<col>: <severity>: <message>
//! ```
//!
//! For example:
//!
//! ```text
//! foo.clj:1:12: warning: unused binding x
//! foo.clj:2:16: error: Unresolved symbol: y
//! ```
//!
//! A summary line like `linting took 10ms, errors: 1, warnings: 0` is also
//! printed, but it does not match the format above and is skipped by the
//! parser. The `<severity>: ` prefix is dropped before the message is stored
//! in the [`Entry`].

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct ClojureCljkondo {
    filename: PathBuf,
    inner: Linter,
}

impl ClojureCljkondo {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let mut cmd = Command::new("clj-kondo");
        cmd.arg("--repro");
        cmd.arg("--cache");
        cmd.arg("false");
        cmd.arg("--lint");
        cmd.arg(filename);
        let inner = linters.spawn("clj-kondo", cmd)?;
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
        if parts.len() < 5 {
            return None;
        }
        let line_num: u32 = parts[1].trim().parse().ok()?;
        let col_num: u32 = parts[2].trim().parse().ok()?;
        let msg = parts[4].trim();
        Some(Entry::new_line_col(filename, "clj-kondo", msg, line_num, col_num).unwrap())
    }
}

impl Stream for ClojureCljkondo {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "clj-kondo",
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
        let entry = ClojureCljkondo::parse_line(
            Path::new("foo.clj"),
            "foo.clj:1:12: warning: unused binding x",
        )
        .unwrap();
        assert_eq!(entry.to_string(), "foo.clj:1: [clj-kondo] unused binding x");
    }

    #[test]
    fn parse_line_skips_summary() {
        assert!(
            ClojureCljkondo::parse_line(
                Path::new("foo.clj"),
                "linting took 10ms, errors: 1, warnings: 0"
            )
            .is_none()
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(ClojureCljkondo::parse_line(Path::new("foo.clj"), "").is_none());
    }

    #[test]
    fn parse_line_too_few_parts() {
        assert!(ClojureCljkondo::parse_line(Path::new("foo.clj"), "no colons here").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(
            ClojureCljkondo::parse_line(Path::new("foo.clj"), "foo.clj:x:y: warning: msg")
                .is_none()
        );
    }
}
