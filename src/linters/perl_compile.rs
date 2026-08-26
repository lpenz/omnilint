// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [Perl](https://www.perl.org/) compile checker.
//!
//! `perl -c` checks a Perl script for compilation errors without executing
//! it. It serves as a basic fallback alongside perlcritic.
//!
//! ## Output format
//!
//! Syntax errors are printed to stderr:
//!
//! ```text
//! syntax error at <filename> line N, at EOF
//! <filename>, line N: <message>
//! ```
//!
//! On success, it prints:
//!
//! ```text
//! <filename> syntax OK
//! ```

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct PerlCompile {
    filename: PathBuf,
    inner: Linter,
}

impl PerlCompile {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("perl");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("-c");
        cmd.arg(filename);
        let inner = linters.spawn("perl", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        let line = line.trim();
        let fname = filename.to_str()?;
        let marker = format!("{fname}, line ");
        let rest = line.strip_prefix(&marker)?;
        let (line_str, rest) = rest.split_once(':')?;
        let line_num: u32 = line_str.parse().ok()?;
        let msg = rest.trim();
        Some(Entry::new_line(filename, "perl-compile", msg, line_num).unwrap())
    }
}

impl Stream for PerlCompile {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "perl",
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
        let entry =
            PerlCompile::parse_line(Path::new("test.pl"), "test.pl, line 5: Missing semicolon")
                .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.pl:5: [perl-compile] Missing semicolon"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(PerlCompile::parse_line(Path::new("test.pl"), "").is_none());
    }

    #[test]
    fn parse_line_syntax_ok() {
        assert!(PerlCompile::parse_line(Path::new("test.pl"), "test.pl syntax OK").is_none());
    }
}
