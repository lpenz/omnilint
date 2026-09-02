// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [cppcheck](https://cppcheck.sourceforge.io/) C/C++ linter wrapper.
//!
//! cppcheck checks C and C++ files for bugs. It is run once per file with
//! `--quiet --enable=warning` so that only actual findings are printed, but
//! no style or performance noise, analysing a single file and creating no
//! build artifacts.
//!
//! ## Output format
//!
//! Each finding emitted by cppcheck on stderr has the form:
//!
//! ```text
//! <filename>:<line>:<col>: <severity>: <message> [<id>]
//! ```
//!
//! For example:
//!
//! ```text
//! c-dirty.c:5:3: error: Memory leak: p [memleak]
//! ```
//!
//! cppcheck also prints the offending source line and a caret below the
//! finding; those context lines do not match the format above and are skipped
//! by the parser. The severity prefix is discarded, keeping the message.

use crate::entry::Entry;
use crate::linters::{CommandLinter, Linters, Spec, into_entries};

use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio_stream::Stream;

pub struct CCppcheck(CommandLinter);

impl CCppcheck {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        Ok(Self(CommandLinter::new(
            linters,
            Spec {
                name: "cppcheck",
                args: &["--quiet", "--enable=warning"],
                findings_on_stderr: true,
                parse: |f, l| into_entries(f, l, Self::parse_line),
                ..Default::default()
            },
            filename,
        )?))
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }
        let (loc, rest) = line.split_once(": ")?;
        let (_, msg) = rest.split_once(": ")?;
        let mut parts = loc.split(':');
        let _file = parts.next()?;
        let line_num: u32 = parts.next()?.parse().ok()?;
        let col_num: u32 = parts.next()?.parse().ok()?;
        if parts.next().is_some() {
            return None;
        }
        Some(Entry::new_line_col(filename, "cppcheck", msg, line_num, col_num).unwrap())
    }
}

linter_stream!(CCppcheck);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_standard() {
        let entry = CCppcheck::parse_line(
            Path::new("foo.c"),
            "foo.c:5:3: error: Memory leak: p [memleak]",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "foo.c:5: [cppcheck] Memory leak: p [memleak]"
        );
    }

    #[test]
    fn parse_line_skips_context() {
        assert!(CCppcheck::parse_line(Path::new("foo.c"), "  return 0;").is_none());
        assert!(CCppcheck::parse_line(Path::new("foo.c"), "^").is_none());
    }

    #[test]
    fn parse_line_empty() {
        assert!(CCppcheck::parse_line(Path::new("foo.c"), "").is_none());
    }

    #[test]
    fn parse_line_unparseable() {
        assert!(CCppcheck::parse_line(Path::new("foo.c"), "garbage").is_none());
    }
}
