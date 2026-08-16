// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [ktlint](https://pinterest.github.io/ktlint/) Kotlin linter wrapper.
//!
//! ktlint checks Kotlin files for style and formatting issues following the
//! official Kotlin style guide. It is run once per file, analysing a single
//! file and creating no build artifacts.
//!
//! ## Output format
//!
//! Each finding emitted by ktlint on stdout has the form:
//!
//! ```text
//! <filename>:<line>:<col>: <message> (<rule>)
//! ```
//!
//! For example:
//!
//! ```text
//! Foo.kt:2:21: Unnecessary semicolon (standard:no-semi)
//! ```
//!
//! The `<rule>` suffix is kept as part of the message. ktlint also prints a
//! summary with the error count and, when findings can be autocorrected, a
//! warning line with a timestamp prefix; neither matches the format above and
//! both are skipped by the parser.

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct KotlinKtlint {
    filename: PathBuf,
    inner: Linter,
}

impl KotlinKtlint {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let mut cmd = Command::new("ktlint");
        cmd.arg(filename);
        let inner = linters.spawn("ktlint", cmd)?;
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
        let parts: Vec<&str> = line.splitn(4, ':').collect();
        if parts.len() < 4 {
            return None;
        }
        let line_num: u32 = parts.get(1)?.trim().parse().ok()?;
        let col_num: u32 = parts.get(2)?.trim().parse().ok()?;
        let msg = parts.get(3)?.trim();
        Some(Entry::new_line_col(filename, "ktlint", msg, line_num, col_num).unwrap())
    }
}

impl Stream for KotlinKtlint {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "ktlint",
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
        let entry = KotlinKtlint::parse_line(
            Path::new("Foo.kt"),
            "Foo.kt:2:21: Unnecessary semicolon (standard:no-semi)",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "Foo.kt:2: [ktlint] Unnecessary semicolon (standard:no-semi)"
        );
    }

    #[test]
    fn parse_line_skips_warning() {
        assert!(
            KotlinKtlint::parse_line(
                Path::new("Foo.kt"),
                "11:46:12.149 [main] WARN com.pinterest.ktlint.cli.internal.KtlintCommandLine -- Lint has found errors",
            )
            .is_none()
        );
    }

    #[test]
    fn parse_line_skips_summary() {
        assert!(KotlinKtlint::parse_line(Path::new("Foo.kt"), "  standard:no-semi: 1").is_none());
        assert!(
            KotlinKtlint::parse_line(
                Path::new("Foo.kt"),
                "Summary error count (descending) by rule:"
            )
            .is_none()
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(KotlinKtlint::parse_line(Path::new("Foo.kt"), "").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(KotlinKtlint::parse_line(Path::new("Foo.kt"), "Foo.kt:x:y: msg").is_none());
    }
}
