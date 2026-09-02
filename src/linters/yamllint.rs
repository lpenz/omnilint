// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [yamllint](https://yamllint.readthedocs.io/) YAML linter wrapper.
//!
//! yamllint checks YAML files for syntax errors, formatting issues, and
//! custom rule violations. It is run once per file with `-f parsable` to
//! produce machine-readable output that is parsed into [`Entry`] values.
//!
//! ## Output format
//!
//! Each line emitted by yamllint on stdout has the form:
//!
//! ```text
//! <filename>:<line>:<col>: [<severity>] <message> (<rule-id>)
//! ```
//!
//! For example:
//!
//! ```text
//! config.yaml:1:1: [warning] missing document start "---" (document-start)
//! config.yaml:5:1: [error] wrong indentation: expected 2 but found 4 (indentation)
//! ```
//!
//! The `[<severity>]` prefix and `(<rule-id>)` suffix are stripped by
//! the parser before the message is stored in the [`Entry`].

use crate::entry::Entry;
use crate::linters::{CommandLinter, Linters, Spec, into_entries};

use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio_stream::Stream;

pub struct YamlYamllint(CommandLinter);

impl YamlYamllint {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        Ok(Self(CommandLinter::new(
            linters,
            Spec {
                name: "yamllint",
                args: &["-f", "parsable"],
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
        let parts: Vec<&str> = line.splitn(4, ':').collect();
        assert!(parts.len() >= 4, "unexpected yamllint output: {line}");
        let line_num: u32 = parts[1].trim().parse().ok()?;
        let col_num: u32 = parts[2].trim().parse().ok()?;
        let raw_msg = parts[3].trim();
        // Strip the [severity] prefix and (rule-id) suffix
        let msg = raw_msg
            .strip_prefix('[')
            .and_then(|s| s.find(']').map(|i| s[i + 1..].trim()))
            .unwrap_or(raw_msg);
        let msg = msg.rsplit_once(" (").map_or(msg, |(before, _)| before);
        Some(Entry::new_line_col(filename, "yamllint", msg, line_num, col_num).unwrap())
    }
}

linter_stream!(YamlYamllint);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_standard() {
        let entry = YamlYamllint::parse_line(
            Path::new("test.yaml"),
            "test.yaml:1:1: [warning] missing document start \"---\" (document-start)",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.yaml:1: [yamllint] missing document start \"---\""
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(YamlYamllint::parse_line(Path::new("test.yaml"), "").is_none());
    }

    #[test]
    #[should_panic(expected = "unexpected yamllint output")]
    fn parse_line_too_few_parts() {
        YamlYamllint::parse_line(Path::new("test.yaml"), "no colons here");
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(
            YamlYamllint::parse_line(Path::new("test.yaml"), "test.yaml:x:y: [error] msg (rule)")
                .is_none()
        );
    }
}
