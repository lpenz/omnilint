// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [php](https://www.php.net/) syntax linter wrapper using `php -l`.
//!
//! `php -l` checks that a PHP file parses without errors, without
//! executing it.
//!
//! ## Output format
//!
//! Each error is a single line emitted on stdout, with the form:
//!
//! ```text
//! Parse error: <message> in <filename> on line <line>
//! Errors parsing <filename>
//! ```
//!
//! For example:
//!
//! ```text
//! Parse error: syntax error, unexpected identifier "bar" in foo.php on line 3
//! ```

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct PhpLint {
    filename: PathBuf,
    inner: Linter,
}

impl PhpLint {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("php");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("-l");
        cmd.arg(filename);
        let inner = linters.spawn("php", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        let line = line.trim();
        // Older PHP versions prefix the message with "PHP " and use two
        // spaces after the colon; both forms are accepted.
        let rest = line.strip_prefix("PHP ").unwrap_or(line);
        let rest = rest.strip_prefix("Parse error:")?.trim_start();
        let idx = rest.rfind(" in ")?;
        let msg = rest[..idx].trim_end();
        if msg.is_empty() {
            return None;
        }
        let tail = rest[idx + " in ".len()..].trim();
        let lidx = tail.rfind(" on line ")?;
        let num: u32 = tail[lidx + " on line ".len()..].trim().parse().ok()?;
        Some(Entry::new_line(filename, "php", msg, num).unwrap())
    }
}

impl Stream for PhpLint {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "php",
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
        let entry = PhpLint::parse_line(
            Path::new("test.php"),
            r#"Parse error: syntax error, unexpected identifier "bar" in test.php on line 3"#,
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            r#"test.php:3: [php] syntax error, unexpected identifier "bar""#
        );
    }

    #[test]
    fn parse_line_php_prefix() {
        let entry = PhpLint::parse_line(
            Path::new("test.php"),
            r#"PHP Parse error:  syntax error, unexpected ';' in /tmp/test.php on line 12"#,
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            r#"test.php:12: [php] syntax error, unexpected ';'"#
        );
    }

    #[test]
    fn parse_line_skips_errors_parsing() {
        assert!(PhpLint::parse_line(Path::new("test.php"), "Errors parsing test.php").is_none());
    }

    #[test]
    fn parse_line_empty() {
        assert!(PhpLint::parse_line(Path::new("test.php"), "").is_none());
    }

    #[test]
    fn parse_line_no_line_number() {
        assert!(
            PhpLint::parse_line(Path::new("test.php"), "Parse error: oops in test.php").is_none()
        );
    }
}
