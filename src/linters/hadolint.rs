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
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct DockerfileHadolint {
    filename: PathBuf,
    inner: Linter,
}

impl DockerfileHadolint {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let mut cmd = Command::new("hadolint");
        cmd.arg("--no-color");
        cmd.arg(filename);
        let inner = linters.spawn("hadolint", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
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

impl Stream for DockerfileHadolint {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "hadolint",
            &this.filename,
            &mut this.inner,
            Self::parse_line,
            cx,
        )
    }
}

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
