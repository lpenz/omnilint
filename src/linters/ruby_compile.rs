// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [Ruby](https://www.ruby-lang.org/) compile checker.
//!
//! `ruby -c` checks a Ruby file for syntax errors. It serves as a basic
//! fallback alongside rubocop and standardrb.
//!
//! ## Output format
//!
//! Syntax errors are printed to stderr:
//!
//! ```text
//! <filename>:<line>: syntax error, <message>
//! ```
//!
//! On success, it prints:
//!
//! ```text
//! Syntax OK
//! ```

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct RubyCompile {
    filename: PathBuf,
    inner: Linter,
}

impl RubyCompile {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("ruby");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("-c");
        cmd.arg(filename);
        let inner = linters.spawn("ruby-compile", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        let line = line.trim();
        let fname = filename.to_str()?;
        let marker = format!("{fname}:");
        let rest = line.strip_prefix(&marker)?;
        let (line_str, rest) = rest.split_once(':')?;
        let line_num: u32 = line_str.parse().ok()?;
        let msg = rest.trim().strip_prefix("syntax error, ")?;
        Some(Entry::new_line(filename, "ruby-compile", msg, line_num).unwrap())
    }
}

impl Stream for RubyCompile {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "ruby-compile",
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
            RubyCompile::parse_line(Path::new("test.rb"), "test.rb:3: syntax error, unexpected")
                .unwrap();
        assert_eq!(entry.to_string(), "test.rb:3: [ruby-compile] unexpected");
    }

    #[test]
    fn parse_line_syntax_ok() {
        assert!(RubyCompile::parse_line(Path::new("test.rb"), "Syntax OK").is_none());
    }

    #[test]
    fn parse_line_empty() {
        assert!(RubyCompile::parse_line(Path::new("test.rb"), "").is_none());
    }

    #[test]
    fn parse_line_no_syntax_error() {
        assert!(RubyCompile::parse_line(Path::new("test.rb"), "test.rb:3: other error").is_none());
    }
}
