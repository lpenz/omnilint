// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [jq](https://jqlang.github.io/jq/) JSON linter wrapper.
//!
//! jq checks JSON files for well-formedness. It is run once per file with the
//! `empty` filter so that no data is printed on stdout, analysing a single
//! file and creating no build artifacts.
//!
//! ## Output format
//!
//! Each finding emitted by jq on stderr has the form:
//!
//! ```text
//! jq: parse error: <message> at line <line>, column <col>
//! ```
//!
//! For example:
//!
//! ```text
//! jq: parse error: Unmatched '}' at line 1, column 15
//! ```
//!
//! The trailing location is also present in the entry, so it is stripped from
//! the message.

use crate::entry::Entry;
use crate::linters::{CommandLinter, Linters, Spec, into_entries};

use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio_stream::Stream;

pub struct JsonJq(CommandLinter);

impl JsonJq {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        Ok(Self(CommandLinter::new(
            linters,
            Spec {
                name: "jq",
                args: &["empty"],
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
        let rest = line.strip_prefix("jq: parse error: ")?;
        let (msg, rest) = rest.split_once(" at line ")?;
        let (line_num, col_num) = rest.split_once(", column ")?;
        let line_num: u32 = line_num.parse().ok()?;
        let col_num: u32 = col_num.parse().ok()?;
        Some(Entry::new_line_col(filename, "jq", msg, line_num, col_num).unwrap())
    }
}

linter_stream!(JsonJq);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_standard() {
        let entry = JsonJq::parse_line(
            Path::new("foo.json"),
            "jq: parse error: Unmatched '}' at line 1, column 15",
        )
        .unwrap();
        assert_eq!(entry.to_string(), "foo.json:1: [jq] Unmatched '}'");
    }

    #[test]
    fn parse_line_empty() {
        assert!(JsonJq::parse_line(Path::new("foo.json"), "").is_none());
    }

    #[test]
    fn parse_line_unparseable() {
        assert!(JsonJq::parse_line(Path::new("foo.json"), "garbage").is_none());
    }
}
