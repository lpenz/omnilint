// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [hadolint](https://github.com/hadolint/hadolint) Dockerfile linter
//! wrapper.
//!
//! hadolint lints Dockerfiles for errors, warnings and best practices. It is
//! run once per file with colors disabled, analysing a single file and
//! creating no build artifacts.
//!
//! ## Output format
//!
//! Each finding emitted by hadolint on stdout has the form:
//!
//! ```text
//! <filename>:<line> <code> <severity>: <message>
//! ```
//!
//! For example:
//!
//! ```text
//! Dockerfile:1 DL3007 warning: Using latest is prone to errors if the image will ever update.
//! ```
//!
//! The `<code> <severity>` part between the line number and the message is
//! dropped by the parser before the message is stored in the [`Entry`].

use crate::entry::Entry;
use crate::linters::{CommandLinter, Linters, Spec, into_entries};

use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio_stream::Stream;

pub struct DockerfileHadolint(CommandLinter);

impl DockerfileHadolint {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        Ok(Self(CommandLinter::new(
            linters,
            Spec {
                name: "hadolint",
                args: &["--no-color"],
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
        let parts: Vec<&str> = line.splitn(3, ':').collect();
        let line_num: u32 = parts.get(1)?.split_whitespace().next()?.parse().ok()?;
        let msg = parts.get(2)?.trim();
        Some(Entry::new_line(filename, "hadolint", msg, line_num).unwrap())
    }
}

linter_stream!(DockerfileHadolint);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_standard() {
        let entry = DockerfileHadolint::parse_line(
            Path::new("Dockerfile"),
            "Dockerfile:1 DL3007 warning: Using latest is prone to errors",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "Dockerfile:1: [hadolint] Using latest is prone to errors"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(DockerfileHadolint::parse_line(Path::new("Dockerfile"), "").is_none());
    }

    #[test]
    fn parse_line_too_few_parts() {
        assert!(
            DockerfileHadolint::parse_line(Path::new("Dockerfile"), "no colons here").is_none()
        );
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(
            DockerfileHadolint::parse_line(
                Path::new("Dockerfile"),
                "Dockerfile:x DL1000 error: msg"
            )
            .is_none()
        );
    }
}
