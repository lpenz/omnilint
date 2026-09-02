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
use crate::linters::{CommandLinter, Executable, Linters, Spec, parse_line_standard};

use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio_stream::Stream;

pub struct GoGovet(CommandLinter);

impl GoGovet {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        Ok(Self(CommandLinter::new(
            linters,
            Spec {
                name: "go-vet",
                args: &["vet"],
                findings_on_stderr: true,
                exec: Executable::Mapped,
                parse: parse_line,
                ..Default::default()
            },
            filename,
        )?))
    }
}

fn parse_line(filename: &Path, line: &str) -> Vec<Entry> {
    parse_line_standard(filename, "go-vet", line)
}

linter_stream!(GoGovet);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_standard() {
        let entries = parse_line(Path::new("test.go"), "test.go:10:2: unreachable code");
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].to_string(),
            "test.go:10: [go-vet] unreachable code"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(parse_line(Path::new("test.go"), "").is_empty());
    }

    #[test]
    fn parse_line_too_few_parts() {
        assert!(parse_line(Path::new("test.go"), "no colons here").is_empty());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(parse_line(Path::new("test.go"), "test.go:x:y: msg").is_empty());
    }
}
