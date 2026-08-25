// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [GHDL](https://github.com/ghdl/ghdl) VHDL linter wrapper.
//!
//! GHDL is run once per file with `-a` (analyze), so that it only checks
//! the file for syntax and semantic errors and creates no simulation
//! artifacts.
//!
//! ## Output format
//!
//! Each finding emitted by GHDL on stderr has the form:
//!
//! ```text
//! <filename>:<line>:<col>:<severity>: <message>
//! ```
//!
//! For example:
//!
//! ```text
//! test.vhd:13:9:error: port "a" can't be assigned
//! ```
//!
//! When stderr is not a terminal (the case when run under omnilint), the
//! severity prefix is omitted:
//!
//! ```text
//! test.vhd:13:9: port "a" can't be assigned
//! ```
//!
//! GHDL also prints source context lines and caret markers after each
//! finding; those lines do not match the location format and are
//! silently skipped by the parser.

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct VhdlGhdl {
    filename: PathBuf,
    inner: Linter,
}

impl VhdlGhdl {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("ghdl");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("-a");
        cmd.arg(filename);
        let inner = linters.spawn("ghdl", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        let line = line.trim();
        // Format: filename:line:col:[error|warning:] message
        // Split on ':' to extract fields.
        let parts: Vec<&str> = line.splitn(5, ':').collect();
        // We need at least 4 parts: filename, line, col, and the rest
        if parts.len() < 4 {
            return None;
        }
        // The filename part must end with the base filename to filter out
        // findings about other files (e.g. dependencies).
        let base = filename.file_name()?.to_str()?;
        if !parts[0].ends_with(base) {
            return None;
        }
        let line_num: u32 = parts.get(1)?.parse().ok()?;
        let _col_num: u32 = parts.get(2)?.parse().ok()?;
        // The message may or may not have a severity prefix ("error:" or
        // "warning:").  Strip it if present.
        let msg = match *parts.get(3)? {
            "error" | "warning" => parts.get(4)?.trim(),
            other => other.trim(),
        };
        if msg.is_empty() {
            return None;
        }
        Entry::new_line(filename, "ghdl", msg, line_num).ok()
    }
}

impl Stream for VhdlGhdl {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "ghdl",
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
    fn parse_line_with_severity() {
        let entry = VhdlGhdl::parse_line(
            Path::new("test.vhd"),
            "test.vhd:13:9:error: port \"a\" can't be assigned",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.vhd:13: [ghdl] port \"a\" can't be assigned"
        );
    }

    #[test]
    fn parse_line_without_severity() {
        let entry = VhdlGhdl::parse_line(
            Path::new("test.vhd"),
            "test.vhd:13:9: port \"a\" can't be assigned",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.vhd:13: [ghdl] port \"a\" can't be assigned"
        );
    }

    #[test]
    fn parse_line_warning() {
        let entry = VhdlGhdl::parse_line(
            Path::new("test.vhdl"),
            "test.vhdl:5:1:warning: signal \"x\" is read but never assigned",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.vhdl:5: [ghdl] signal \"x\" is read but never assigned"
        );
    }

    #[test]
    fn parse_line_discards_other_files() {
        assert!(
            VhdlGhdl::parse_line(
                Path::new("test.vhd"),
                "other.vhd:10:5:error: undeclared identifier"
            )
            .is_none()
        );
    }

    #[test]
    fn parse_line_skips_context_lines() {
        assert!(VhdlGhdl::parse_line(Path::new("test.vhd"), "        a <= b;").is_none());
        assert!(VhdlGhdl::parse_line(Path::new("test.vhd"), "        ^").is_none());
    }

    #[test]
    fn parse_line_empty() {
        assert!(VhdlGhdl::parse_line(Path::new("test.vhd"), "").is_none());
    }
}
