// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [phpmd](https://phpmd.org/) PHP Mess Detector wrapper.
//!
//! phpmd checks PHP source files for potential problems such as unused
//! code, overly complex structures and naming issues. It is run with the
//! `text` renderer over the default rule sets, producing output that is
//! parsed into [`Entry`] values.
//!
//! ## Output format
//!
//! Each violation is a single line emitted on stdout, with the form:
//!
//! ```text
//! <filename>:<line>  <rule name>  <message>
//! ```
//!
//! where the rule name is followed by padding spaces so that the messages
//! are aligned. The rule name and message are joined with ": " in the
//! resulting [`Entry`].

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

const RULESETS: &str = "cleancode,codesize,controversial,design,naming,unusedcode";

pub struct PhpPhpmd {
    filename: PathBuf,
    inner: Linter,
}

impl PhpPhpmd {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("phpmd");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg(filename);
        cmd.arg("text");
        cmd.arg(RULESETS);
        let inner = linters.spawn("phpmd", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        let line = line.trim();
        let idx = line.find(':')?;
        let rest = &line[idx + 1..];
        let num_end = rest.find(|c: char| !c.is_ascii_digit())?;
        if num_end == 0 {
            return None;
        }
        let num: u32 = rest[..num_end].parse().ok()?;
        let msg = rest[num_end..].trim();
        if msg.is_empty() {
            return None;
        }
        // Normalize the padded rule-name column into a ": " separator
        let msg = match msg.split_once("  ") {
            Some((rule, tail)) => {
                let tail = tail.trim_start();
                if tail.is_empty() {
                    rule.to_string()
                } else {
                    format!("{rule}: {tail}")
                }
            }
            None => msg.to_string(),
        };
        Some(Entry::new_line(filename, "phpmd", &msg, num).unwrap())
    }
}

impl Stream for PhpPhpmd {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "phpmd",
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
        let entry = PhpPhpmd::parse_line(
            Path::new("test.php"),
            "/tmp/test.php:7  UnusedLocalVariable  Avoid unused local variables such as '$unused'.",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.php:7: [phpmd] UnusedLocalVariable: Avoid unused local variables such as '$unused'."
        );
    }

    #[test]
    fn parse_line_padded_rule_name() {
        let entry = PhpPhpmd::parse_line(
            Path::new("test.php"),
            "/tmp/test.php:7  ShortVariable        Avoid variables with short names like $x.",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.php:7: [phpmd] ShortVariable: Avoid variables with short names like $x."
        );
    }

    #[test]
    fn parse_line_no_space_message() {
        let entry =
            PhpPhpmd::parse_line(Path::new("test.php"), "test.php:12Avoid short names").unwrap();
        assert_eq!(entry.to_string(), "test.php:12: [phpmd] Avoid short names");
    }

    #[test]
    fn parse_line_skips_deprecation_noise() {
        assert!(
            PhpPhpmd::parse_line(
                Path::new("test.php"),
                "Deprecated: PHPMD\\Rule::isValidPropertyNode(): Implicitly marking parameter as nullable is deprecated in /x.php on line 150"
            )
            .is_none()
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(PhpPhpmd::parse_line(Path::new("test.php"), "").is_none());
    }

    #[test]
    fn parse_line_no_colon() {
        assert!(PhpPhpmd::parse_line(Path::new("test.php"), "no colon here").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(PhpPhpmd::parse_line(Path::new("test.php"), "test.php:x  msg").is_none());
    }
}
