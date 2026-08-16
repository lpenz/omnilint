// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [xmllint](https://gitlab.gnome.org/GNOME/libxml2/-/wikis/home) XML linter wrapper.
//!
//! xmllint checks XML files for well-formedness. It is run once per file with
//! `--noout` so that the parsed tree is not printed, analysing a single file
//! and creating no build artifacts.
//!
//! ## Output format
//!
//! Each finding emitted by xmllint on stderr has the form:
//!
//! ```text
//! <filename>:<line>: parser error : <message>
//! ```
//!
//! For example:
//!
//! ```text
//! xml-dirty.xml:3: parser error : expected '>'
//! ```
//!
//! xmllint also prints the offending source line and a caret below the
//! finding; those context lines do not match the format above and are skipped
//! by the parser.

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct XmlXmllint {
    filename: PathBuf,
    inner: Linter,
}

impl XmlXmllint {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let mut cmd = Command::new("xmllint");
        cmd.arg("--noout");
        cmd.arg(filename);
        let inner = linters.spawn("xmllint", cmd)?;
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
        let (loc, msg) = line.split_once(": parser error : ")?;
        let (_, line_num) = loc.rsplit_once(':')?;
        let line_num: u32 = line_num.parse().ok()?;
        Some(Entry::new_line(filename, "xmllint", msg, line_num).unwrap())
    }
}

impl Stream for XmlXmllint {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "xmllint",
            &this.filename,
            &mut this.inner,
            Self::parse_line,
            true,
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_standard() {
        let entry = XmlXmllint::parse_line(
            Path::new("foo.xml"),
            "foo.xml:3: parser error : expected '>'",
        )
        .unwrap();
        assert_eq!(entry.to_string(), "foo.xml:3: [xmllint] expected '>'");
    }

    #[test]
    fn parse_line_skips_context() {
        assert!(XmlXmllint::parse_line(Path::new("foo.xml"), "</root>").is_none());
        assert!(XmlXmllint::parse_line(Path::new("foo.xml"), "^").is_none());
    }

    #[test]
    fn parse_line_empty() {
        assert!(XmlXmllint::parse_line(Path::new("foo.xml"), "").is_none());
    }

    #[test]
    fn parse_line_unparseable() {
        assert!(XmlXmllint::parse_line(Path::new("foo.xml"), "garbage").is_none());
    }
}
