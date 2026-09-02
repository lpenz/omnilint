// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [protolint](https://github.com/yoheimuta/protolint) Protobuf linter wrapper.
//!
//! protolint checks Protocol Buffer files for style and correctness. It is
//! run once per file, analysing a single file and creating no build artifacts.
//!
//! ## Output format
//!
//! Each finding emitted by protolint on stderr has the form:
//!
//! ```text
//! [<filename>:<line>:<col>] <message>
//! ```
//!
//! For example:
//!
//! ```text
//! [proto_dirty.proto:4:2] Found an incorrect indentation style "\t". "  " is correct.
//! ```

use crate::entry::Entry;
use crate::linters::{CommandLinter, Linters, Spec, into_entries};

use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio_stream::Stream;

pub struct ProtoProtolint(CommandLinter);

impl ProtoProtolint {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        Ok(Self(CommandLinter::new(
            linters,
            Spec {
                name: "protolint",
                args: &["lint"],
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
        let rest = line.strip_prefix('[')?;
        let (loc, msg) = rest.split_once("] ")?;
        let mut parts = loc.split(':');
        let _file = parts.next()?;
        let line_num: u32 = parts.next()?.parse().ok()?;
        let col_num: u32 = parts.next()?.parse().ok()?;
        if parts.next().is_some() {
            return None;
        }
        Some(Entry::new_line_col(filename, "protolint", msg, line_num, col_num).unwrap())
    }
}

linter_stream!(ProtoProtolint);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_standard() {
        let entry = ProtoProtolint::parse_line(
            Path::new("foo.proto"),
            "[foo.proto:4:2] Found an incorrect indentation style \"\t\". \"  \" is correct.",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "foo.proto:4: [protolint] Found an incorrect indentation style \"\t\". \"  \" is correct."
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(ProtoProtolint::parse_line(Path::new("foo.proto"), "").is_none());
    }

    #[test]
    fn parse_line_unparseable() {
        assert!(ProtoProtolint::parse_line(Path::new("foo.proto"), "garbage").is_none());
    }
}
