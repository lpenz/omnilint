// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [salt-lint](https://github.com/warpnet/salt-lint) SaltStack linter
//! wrapper.
//!
//! salt-lint checks SaltStack state files (`*.sls`) against a set of rules,
//! such as Jinja variable spacing and deprecated features. Its default
//! output spans multiple lines per finding, so the lines are buffered into
//! [`Entry`] values.
//!
//! ## Output format
//!
//! Each finding consists of three consecutive lines on stdout:
//!
//! ```text
//! [<code>] <message>
//! <filename>:<line>
//!     <source line>
//! ```
//!
//! For example:
//!
//! ```text
//! [206] Jinja variables should have spaces before and after: '{{ var_name }}'
//! foo.sls:3
//!     installed: {{var}}
//! ```

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct SaltSaltlint {
    filename: PathBuf,
    inner: Linter,
    /// Message of the finding whose location line has not been seen yet.
    pending: Option<String>,
}

impl SaltSaltlint {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("salt-lint");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg(filename);
        let inner = linters.spawn("salt-lint", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
            pending: None,
        })
    }

    fn parse_line(pending: &mut Option<String>, filename: &Path, line: &str) -> Option<Entry> {
        let line = line.trim_end();
        let trimmed = line.trim_start();
        if trimmed.starts_with('[')
            && let Some(end) = trimmed.find(']')
            && end > 1
            && trimmed[1..end].chars().all(|c| c.is_ascii_digit())
        {
            // "[<code>] <message>": buffer it until the location line comes
            *pending = Some(trimmed.to_string());
            return None;
        }
        if let Some(message) = pending.take()
            && let Some(idx) = line.rfind(':')
        {
            let num = &line[idx + 1..];
            if !num.is_empty() && num.chars().all(|c| c.is_ascii_digit()) {
                return Some(
                    Entry::new_line(filename, "salt-lint", &message, num.parse().ok()?).unwrap(),
                );
            }
        }
        None
    }
}

impl Stream for SaltSaltlint {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let SaltSaltlint {
            filename,
            inner,
            pending,
        } = self.get_mut();
        crate::linters::poll_next(
            "salt-lint",
            filename.as_path(),
            inner,
            |filename, line| Self::parse_line(pending, filename, line),
            false,
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(pending: &mut Option<String>, line: &str) -> Option<Entry> {
        SaltSaltlint::parse_line(pending, Path::new("test.sls"), line)
    }

    #[test]
    fn parse_finding_pair() {
        let mut pending = None;
        assert!(
            parse(
                &mut pending,
                "[206] Jinja variables should have spaces before and after: '{{ var_name }}'"
            )
            .is_none()
        );
        let entry = parse(&mut pending, "/tmp/test.sls:3").unwrap();
        assert_eq!(
            entry.to_string(),
            "test.sls:3: [salt-lint] [206] Jinja variables should have spaces before and after: '{{ var_name }}'"
        );
        assert!(pending.is_none());
    }

    #[test]
    fn parse_skips_snippet_lines() {
        let mut pending = None;
        parse(&mut pending, "[206] Some message");
        assert!(parse(&mut pending, "    installed: {{var}}").is_none());
        assert!(parse(&mut pending, "").is_none());
        assert!(pending.is_none());
    }

    #[test]
    fn parse_location_without_pending_is_ignored() {
        let mut pending = None;
        assert!(parse(&mut pending, "/tmp/test.sls:3").is_none());
    }

    #[test]
    fn parse_message_resets_on_new_message() {
        let mut pending = None;
        parse(&mut pending, "[206] First message");
        parse(&mut pending, "[201] Second message");
        assert_eq!(pending.as_deref(), Some("[201] Second message"));
    }

    #[test]
    fn parse_brackets_not_code_ignored() {
        let mut pending = None;
        parse(&mut pending, "[] empty code");
        parse(&mut pending, "[x] not digits");
        assert!(pending.is_none());
        assert!(parse(&mut pending, "/tmp/test.sls:3").is_none());
    }
}
