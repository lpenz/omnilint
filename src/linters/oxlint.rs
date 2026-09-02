// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [oxlint](https://oxc.rs/) JavaScript/TypeScript linter wrapper.
//!
//! oxlint checks JavaScript and TypeScript source files for code quality
//! issues. It is run with `--format=unix` to produce machine-readable
//! output that is parsed into [`Entry`] values.
//!
//! ## Output format
//!
//! Each line emitted by oxlint on stdout has the form:
//!
//! ```text
//! <filename>:<line>:<col>: <message>
//! ```
//!
//! For example:
//!
//! ```text
//! foo.js:1:5: Unexpected var, use let or const instead
//! ```

use crate::entry::Entry;
use crate::linters::{CommandLinter, Linters, Spec, parse_line_standard};

use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio_stream::Stream;

pub struct JsOxlint(CommandLinter);

impl JsOxlint {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        Ok(Self(CommandLinter::new(
            linters,
            Spec {
                name: "oxlint",
                args: &["--format=unix"],
                parse: parse_line,
                ..Default::default()
            },
            filename,
        )?))
    }
}

fn parse_line(filename: &Path, line: &str) -> Vec<Entry> {
    parse_line_standard(filename, "oxlint", line)
}

linter_stream!(JsOxlint);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_standard() {
        let entries = parse_line(
            Path::new("test.js"),
            "test.js:1:5: Unexpected var, use let or const instead",
        );
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].to_string(),
            "test.js:1: [oxlint] Unexpected var, use let or const instead"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(parse_line(Path::new("test.js"), "").is_empty());
    }

    #[test]
    fn parse_line_too_few_parts() {
        assert!(parse_line(Path::new("test.js"), "no colons here").is_empty());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(parse_line(Path::new("test.js"), "test.js:x:y: msg").is_empty());
    }
}
