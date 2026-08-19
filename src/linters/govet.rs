// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [go vet](https://pkg.go.dev/cmd/vet/) Go linter wrapper.
//!
//! go vet checks Go source files for suspicious constructs that are
//! likely to be bugs.
//!
//! ## Output format
//!
//! Each line emitted by go vet on stderr has the form:
//!
//! ```text
//! <filename>:<line>:<col>: <message>
//! ```
//!
//! For example:
//!
//! ```text
//! foo.go:10:2: unreachable code
//! ```

use crate::entry::Entry;
use crate::linters::{Linter, Linters, parse_line_standard};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct GoGovet {
    filename: PathBuf,
    inner: Linter,
}

impl GoGovet {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("go");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("vet");
        cmd.arg(filename);
        let inner = linters.spawn("go-vet", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        parse_line_standard(filename, "go-vet", line)
    }
}

impl Stream for GoGovet {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "go-vet",
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
            GoGovet::parse_line(Path::new("test.go"), "test.go:10:2: unreachable code").unwrap();
        assert_eq!(entry.to_string(), "test.go:10: [go-vet] unreachable code");
    }

    #[test]
    fn parse_line_empty() {
        assert!(GoGovet::parse_line(Path::new("test.go"), "").is_none());
    }

    #[test]
    fn parse_line_too_few_parts() {
        assert!(GoGovet::parse_line(Path::new("test.go"), "no colons here").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(GoGovet::parse_line(Path::new("test.go"), "test.go:x:y: msg").is_none());
    }
}
