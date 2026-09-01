// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [proselint](https://github.com/amperser/proselint) prose linter wrapper.
//!
//! proselint checks English prose for style, grammar and word usage issues.
//! It is run against Markdown files with the `check` subcommand, and its
//! output is parsed into [`Entry`] values.
//!
//! ## Output format
//!
//! Each finding emitted by proselint on stdout has the form:
//!
//! ```text
//! <file>:<line>:<col>: <check>: <message>
//! ```
//!
//! For example:
//!
//! ```text
//! text.md:1:9: uncomparables: Comparison of an uncomparable: 'very unique' is not comparable.
//! ```
//!
//! The check name is kept as a prefix of the message stored in the [`Entry`];
//! the `<file>` part is discarded, since proselint reports the resolved
//! absolute path of the file, which may differ from the path that omnilint
//! was called with.

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct MarkdownProselint {
    filename: PathBuf,
    inner: Linter,
}

impl MarkdownProselint {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("proselint");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("check");
        cmd.arg(filename);
        let inner = linters.spawn("proselint", cmd)?;
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
        let line_num: u32 = parts.get(1)?.trim().parse().ok()?;
        let col_num: u32 = parts.get(2)?.trim().parse().ok()?;
        let check_and_msg = parts.get(3)?.trim();
        let (check, msg) = check_and_msg.split_once(':').unwrap_or((check_and_msg, ""));
        let msg = format!("{check}: {}", msg.trim());
        Some(Entry::new_line_col(filename, "proselint", &msg, line_num, col_num).unwrap())
    }
}

impl Stream for MarkdownProselint {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "proselint",
            &this.filename,
            &mut this.inner,
            Self::parse_line,
            false,
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_standard() {
        let entry = MarkdownProselint::parse_line(
            Path::new("text.md"),
            "text.md:1:9: uncomparables: Comparison of an uncomparable: 'very unique' is not comparable.",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "text.md:1: [proselint] uncomparables: Comparison of an uncomparable: 'very unique' is not comparable."
        );
    }

    #[test]
    fn parse_line_with_dotted_check() {
        let entry = MarkdownProselint::parse_line(
            Path::new("text.md"),
            "text.md:3:47: redundancy.misc.after_the_deadline: Redundancy. Use 'every' instead of 'each and every'.",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "text.md:3: [proselint] redundancy.misc.after_the_deadline: Redundancy. Use 'every' instead of 'each and every'."
        );
    }

    #[test]
    fn parse_line_absolute_path() {
        let entry = MarkdownProselint::parse_line(
            Path::new("text.md"),
            "/home/user/project/text.md:1:9: uncomparables: message",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "text.md:1: [proselint] uncomparables: message"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(MarkdownProselint::parse_line(Path::new("text.md"), "").is_none());
    }

    #[test]
    fn parse_line_unparseable() {
        assert!(MarkdownProselint::parse_line(Path::new("text.md"), "garbage").is_none());
    }
}
