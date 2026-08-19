// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [staticcheck](https://staticcheck.io/) Go linter wrapper.
//!
//! staticcheck performs static analysis and linting for Go source files.
//!
//! ## Output format
//!
//! Each line emitted by staticcheck on stdout has the form:
//!
//! ```text
//! <filename>:<line>:<col>: <message>
//! ```
//!
//! For example:
//!
//! ```text
//! foo.go:10:2: undefined: something
//! ```

use crate::entry::Entry;
use crate::linters::{Linter, Linters, parse_line_standard};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct GoStaticcheck {
    filename: PathBuf,
    inner: Linter,
}

impl GoStaticcheck {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("staticcheck");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg(filename);
        let inner = linters.spawn("staticcheck", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        parse_line_standard(filename, "staticcheck", line)
    }
}

impl Stream for GoStaticcheck {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "staticcheck",
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
        let entry =
            GoStaticcheck::parse_line(Path::new("test.go"), "test.go:10:2: undefined: something")
                .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.go:10: [staticcheck] undefined: something"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(GoStaticcheck::parse_line(Path::new("test.go"), "").is_none());
    }

    #[test]
    fn parse_line_too_few_parts() {
        assert!(GoStaticcheck::parse_line(Path::new("test.go"), "no colons here").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(GoStaticcheck::parse_line(Path::new("test.go"), "test.go:x:y: msg").is_none());
    }
}
