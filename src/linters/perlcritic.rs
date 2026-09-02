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
use crate::linters::{CommandLinter, Linters, Spec, parse_line_standard};

use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio_stream::Stream;

pub struct PerlPerlcritic(CommandLinter);

impl PerlPerlcritic {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        Ok(Self(CommandLinter::new(
            linters,
            Spec {
                name: "perlcritic",
                args: &["--quiet", "--verbose", "%f:%l:%c: %m\n"],
                parse: parse_line,
                ..Default::default()
            },
            filename,
        )?))
    }
}

fn parse_line(filename: &Path, line: &str) -> Vec<Entry> {
    parse_line_standard(filename, "perlcritic", line)
}

linter_stream!(PerlPerlcritic);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_standard() {
        let entries = parse_line(
            Path::new("test.pl"),
            "test.pl:2:1: Code before strictures are enabled",
        );
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].to_string(),
            "test.pl:2: [perlcritic] Code before strictures are enabled"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(parse_line(Path::new("test.pl"), "").is_empty());
    }

    #[test]
    fn parse_line_too_few_parts() {
        assert!(parse_line(Path::new("test.pl"), "no colons here").is_empty());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(parse_line(Path::new("test.pl"), "test.pl:x:y: msg").is_empty());
    }
}
