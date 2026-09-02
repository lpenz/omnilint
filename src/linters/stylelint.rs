// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [stylelint](https://stylelint.io/) CSS/SCSS/Less linter wrapper.
//!
//! stylelint checks CSS, SCSS, and Less source files for style errors.
//! It is run with `--formatter=unix` to produce machine-readable output
//! that is parsed into [`Entry`] values.
//!
//! ## Output format
//!
//! Each line emitted by stylelint on stdout has the form:
//!
//! ```text
//! <filename>:<line>:<col>: <message>
//! ```
//!
//! For example:
//!
//! ```text
//! foo.css:1:1: Unexpected empty block (block-no-empty)
//! ```

use crate::entry::Entry;
use crate::linters::{CommandLinter, Linters, Spec, parse_line_standard};

use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio_stream::Stream;

pub struct CssStylelint(CommandLinter);

impl CssStylelint {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        Ok(Self(CommandLinter::new(
            linters,
            Spec {
                name: "stylelint",
                args: &["--formatter=unix"],
                findings_on_stderr: true,
                parse: parse_line,
                ..Default::default()
            },
            filename,
        )?))
    }
}

fn parse_line(filename: &Path, line: &str) -> Vec<Entry> {
    parse_line_standard(filename, "stylelint", line)
}

linter_stream!(CssStylelint);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_standard() {
        let entries = parse_line(
            Path::new("test.css"),
            "test.css:1:1: Unexpected empty block (block-no-empty)",
        );
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].to_string(),
            "test.css:1: [stylelint] Unexpected empty block (block-no-empty)"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(parse_line(Path::new("test.css"), "").is_empty());
    }

    #[test]
    fn parse_line_too_few_parts() {
        assert!(parse_line(Path::new("test.css"), "no colons here").is_empty());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(parse_line(Path::new("test.css"), "test.css:x:y: msg").is_empty());
    }
}
