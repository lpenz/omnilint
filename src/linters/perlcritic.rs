// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [perlcritic](https://metacpan.org/pod/Perl::Critic) Perl linter wrapper.
//!
//! perlcritic checks Perl source files for style errors, best practices,
//! and other issues. It is run once per file with a verbose format string
//! that produces `file:line:col: message` output, which is parsed into
//! [`Entry`] values reusing the standard parser.
//!
//! ## Output format
//!
//! Each line emitted by perlcritic on stdout has the form:
//!
//! ```text
//! <filename>:<line>:<col>: <message>
//! ```
//!
//! For example:
//!
//! ```text
//! foo.pl:2:1: Code before strictures are enabled
//! ```

use crate::entry::Entry;
use crate::linters::{Linter, Linters, parse_line_standard};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct PerlPerlcritic {
    filename: PathBuf,
    inner: Linter,
}

impl PerlPerlcritic {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let mut cmd = Command::new("perlcritic");
        cmd.arg("--quiet");
        cmd.arg("--verbose");
        cmd.arg("%f:%l:%c: %m\n");
        cmd.arg(filename);
        let inner = linters.spawn("perlcritic", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        parse_line_standard(filename, "perlcritic", line)
    }
}

impl Stream for PerlPerlcritic {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "perlcritic",
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
        let entry = PerlPerlcritic::parse_line(
            Path::new("test.pl"),
            "test.pl:2:1: Code before strictures are enabled",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.pl:2: [perlcritic] Code before strictures are enabled"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(PerlPerlcritic::parse_line(Path::new("test.pl"), "").is_none());
    }

    #[test]
    fn parse_line_too_few_parts() {
        assert!(PerlPerlcritic::parse_line(Path::new("test.pl"), "no colons here").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(PerlPerlcritic::parse_line(Path::new("test.pl"), "test.pl:x:y: msg").is_none());
    }
}
