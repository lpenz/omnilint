// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [yamllint](https://yamllint.readthedocs.io/) YAML linter wrapper.
//!
//! yamllint checks YAML files for syntax errors, formatting issues, and
//! custom rule violations. It is run once per file with `-f parsable` to
//! produce machine-readable output that is parsed into [`Entry`] values.
//!
//! ## Output format
//!
//! Each line emitted by yamllint on stdout has the form:
//!
//! ```text
//! <filename>:<line>:<col>: [<severity>] <message> (<rule-id>)
//! ```
//!
//! For example:
//!
//! ```text
//! config.yaml:1:1: [warning] missing document start "---" (document-start)
//! config.yaml:5:1: [error] wrong indentation: expected 2 but found 4 (indentation)
//! ```
//!
//! The `[<severity>]` prefix and `(<rule-id>)` suffix are stripped by
//! the parser before the message is stored in the [`Entry`].

use crate::entry::Entry;

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll, ready};

use color_eyre::Result;
use tokio::process::Command;
use tokio_process_stream::{Item as ProcessItem, ProcessLineStream};
use tokio_stream::Stream;

pub struct YamlYamllint {
    filename: PathBuf,
    inner: ProcessLineStream,
}

impl YamlYamllint {
    pub fn new(filename: &Path) -> Result<Self> {
        let mut cmd = Command::new("yamllint");
        cmd.arg("-f");
        cmd.arg("parsable");
        cmd.arg(filename);
        let inner = ProcessLineStream::try_from(cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }
        let parts: Vec<&str> = line.splitn(4, ':').collect();
        assert!(parts.len() >= 4, "unexpected yamllint output: {line}");
        let line_num: u32 = parts[1].trim().parse().ok()?;
        let col_num: u32 = parts[2].trim().parse().ok()?;
        let raw_msg = parts[3].trim();
        // Strip the [severity] prefix and (rule-id) suffix
        let msg = raw_msg
            .strip_prefix('[')
            .and_then(|s| s.find(']').map(|i| s[i + 1..].trim()))
            .unwrap_or(raw_msg);
        let msg = msg.rsplit_once(" (").map_or(msg, |(before, _)| before);
        Some(Entry::new_line_col(filename, msg, line_num, col_num).unwrap())
    }
}

impl Stream for YamlYamllint {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        loop {
            match ready!(Pin::new(&mut this.inner).poll_next(cx)) {
                Some(ProcessItem::Stdout(line)) => {
                    if let Some(entry) = Self::parse_line(&this.filename, &line) {
                        return Poll::Ready(Some(entry));
                    }
                }
                Some(ProcessItem::Stderr(line)) => {
                    eprintln!("[yamllint {}] stderr {}", this.filename.display(), line);
                }
                Some(ProcessItem::Done(_)) => {
                    // Exit codes: 0 = clean (or warnings without -s),
                    // 1 = errors, 2 = warnings with --strict.
                    // All output is already on stdout, so we can just ignore it.
                    continue;
                }
                None => return Poll::Ready(None),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_standard() {
        let entry = YamlYamllint::parse_line(
            Path::new("test.yaml"),
            "test.yaml:1:1: [warning] missing document start \"---\" (document-start)",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.yaml:1:1: missing document start \"---\""
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(YamlYamllint::parse_line(Path::new("test.yaml"), "").is_none());
    }

    #[test]
    #[should_panic(expected = "unexpected yamllint output")]
    fn parse_line_too_few_parts() {
        YamlYamllint::parse_line(Path::new("test.yaml"), "no colons here");
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(
            YamlYamllint::parse_line(Path::new("test.yaml"), "test.yaml:x:y: [error] msg (rule)")
                .is_none()
        );
    }
}
