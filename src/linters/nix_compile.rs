// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [nix-instantiate](https://nixos.org/) Nix parse checker.
//!
//! `nix-instantiate --parse` parses a Nix expression without evaluating it,
//! serving as a basic syntax check alongside statix.
//!
//! ## Output format
//!
//! Each finding emitted by nix-instantiate on stderr has the form:
//!
//! ```text
//! error: <message>
//!        at <filename>:<line>:<column>:
//! ```

use crate::entry::Entry;
use crate::linters::{CommandLinter, Linters, Spec, into_entries};

use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio_stream::Stream;

pub struct NixNixInstantiate(CommandLinter);

impl NixNixInstantiate {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        Ok(Self(CommandLinter::new(
            linters,
            Spec {
                name: "nix-instantiate",
                args: &["--parse"],
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
        // Match an "at <path>:<line>:<col>:<something>" snippet anywhere in
        // the line. nix-instantiate reports absolute paths, so we ignore the
        // path part and only require a trailing "<line>:<col>".
        let at_idx = line.find("at ")?;
        let after = &line[at_idx + "at ".len()..];
        let rest = after.strip_suffix(':')?;
        let (path, col_str) = rest.rsplit_once(':')?;
        let (_, line_str) = path.rsplit_once(':')?;
        let line_num: u32 = line_str.parse().ok()?;
        let col_num: u32 = col_str.parse().ok()?;
        // If there is a message before the "at" snippet use it, otherwise the
        // whole location was reported on a line of its own.
        let msg = if at_idx > 0 {
            line[..at_idx].trim().trim_end_matches(',')
        } else {
            "syntax error"
        };
        let msg = msg.strip_prefix("error: ").unwrap_or(msg);
        Some(Entry::new_line_col(filename, "nix-instantiate", msg, line_num, col_num).unwrap())
    }
}

linter_stream!(NixNixInstantiate);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_standard() {
        let entry = NixNixInstantiate::parse_line(
            Path::new("test.nix"),
            "error: syntax error, unexpected IN, at test.nix:3:10:",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.nix:3: [nix-instantiate] syntax error, unexpected IN"
        );
    }

    #[test]
    fn parse_line_multiline_location() {
        let entry =
            NixNixInstantiate::parse_line(Path::new("test.nix"), "       at test.nix:1:19:")
                .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.nix:1: [nix-instantiate] syntax error"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(NixNixInstantiate::parse_line(Path::new("test.nix"), "").is_none());
    }

    #[test]
    fn parse_line_no_at() {
        assert!(NixNixInstantiate::parse_line(Path::new("test.nix"), "error: something").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(
            NixNixInstantiate::parse_line(Path::new("test.nix"), "at test.nix:x:y:z").is_none()
        );
    }
}
