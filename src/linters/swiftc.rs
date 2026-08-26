// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [Swift compiler](https://www.swift.org/) syntax checker.
//!
//! `swiftc -parse` checks a Swift file for syntax errors without generating
//! code. It serves as a basic fallback alongside swiftlint.
//!
//! ## Output format
//!
//! Errors are printed to stderr:
//!
//! ```text
//! <filename>:<line>:<col>: error: <message>
//! ```

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct SwiftSwiftc {
    filename: PathBuf,
    inner: Linter,
}

impl SwiftSwiftc {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("swiftc");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("-parse");
        cmd.arg(filename);
        let inner = linters.spawn("swiftc", cmd)?;
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
        let msg = rest.trim().strip_prefix("error: ")?;
        Some(Entry::new_line(filename, "swiftc", msg, line_num).unwrap())
    }
}

impl Stream for SwiftSwiftc {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "swiftc",
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
        let entry = SwiftSwiftc::parse_line(
            Path::new("test.swift"),
            "test.swift:5:10: error: expected declaration",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.swift:5: [swiftc] expected declaration"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(SwiftSwiftc::parse_line(Path::new("test.swift"), "").is_none());
    }

    #[test]
    fn parse_line_warning() {
        assert!(
            SwiftSwiftc::parse_line(
                Path::new("test.swift"),
                "test.swift:5:10: warning: something"
            )
            .is_none()
        );
    }
}
