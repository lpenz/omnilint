// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [proselint](https://github.com/amperser/proselint) prose linter wrapper.
//!
//! proselint checks plain text files for problems of style: clichés,
//! weasel words, redundancy and other writing issues. It is run with the
//! `check` subcommand, and its output is parsed into [`Entry`] values.
//!
//! ## Output format
//!
//! Each finding is a single line emitted on stdout, with the form:
//!
//! ```text
//! <filename>:<line>:<col>: <check name>: <message>
//! ```
//!
//! For example:
//!
//! ```text
//! foo.txt:1:8: cliches.misc.write_good: 'a dark and stormy night' is a cliché
//! ```

use crate::entry::Entry;
use crate::linters::{Linter, Linters, parse_line_standard};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct TextProselint {
    filename: PathBuf,
    inner: Linter,
}

impl TextProselint {
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
        parse_line_standard(filename, "proselint", line)
    }
}

impl Stream for TextProselint {
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
        let entry = TextProselint::parse_line(
            Path::new("test.txt"),
            "test.txt:1:8: cliches.misc.write_good: 'a dark and stormy night' is a cliché",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.txt:1: [proselint] cliches.misc.write_good: 'a dark and stormy night' is a cliché"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(TextProselint::parse_line(Path::new("test.txt"), "").is_none());
    }

    #[test]
    fn parse_line_too_few_parts() {
        assert!(TextProselint::parse_line(Path::new("test.txt"), "no colons here").is_none());
    }
}
