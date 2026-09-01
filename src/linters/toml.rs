// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Built-in TOML parser linter.
//!
//! Unlike the other linters, this one does not invoke an external binary: it
//! uses the [`toml`] crate to parse the file contents in-process. It emits a
//! single [`Entry`] on the first syntax error, pointing at the responsible
//! line and column of the file.

use crate::cli::LinterMode;
use crate::entry::Entry;
use crate::linters::Linters;

use std::collections::VecDeque;
use std::fs;
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio_stream::Stream;

pub struct TomlTomlParse {
    entries: VecDeque<Entry>,
}

impl TomlTomlParse {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let entries = if linters.resolve_mode("toml-parse") == LinterMode::Disabled {
            VecDeque::new()
        } else {
            parse_file(filename)
        };
        Ok(Self { entries })
    }
}

/// Parses a TOML file, returning one [`Entry`] on the first syntax error, or
/// reporting the read error itself if the file can't be read.
fn parse_file(filename: &Path) -> VecDeque<Entry> {
    let mut entries = VecDeque::new();
    let content = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(error) => {
            if let Ok(entry) = Entry::new(filename, "toml-parse", &format!("cannot read: {error}"))
            {
                entries.push_back(entry);
            }
            return entries;
        }
    };
    entries.extend(lint_content(filename, &content));
    entries
}

/// Returns one [`Entry`] on the first syntax error of `content`, or none if
/// the content is a valid TOML document.
fn lint_content(filename: &Path, content: &str) -> VecDeque<Entry> {
    let mut entries = VecDeque::new();
    if let Err(error) = content.parse::<toml::Value>() {
        let msg = error
            .to_string()
            .lines()
            .next_back()
            .unwrap_or("TOML parse error")
            .trim()
            .to_string();
        let entry = match error
            .span()
            .and_then(|span| offset_to_line_col(content, span.start))
        {
            Some((line, col)) => Entry::new_line_col(filename, "toml-parse", &msg, line, col),
            None => Entry::new(filename, "toml-parse", &msg),
        };
        if let Ok(entry) = entry {
            entries.push_back(entry);
        }
    }
    entries
}

/// Converts a byte offset in `content` into a 1-based `(line, col)` pair.
fn offset_to_line_col(content: &str, offset: usize) -> Option<(u32, u32)> {
    if offset > content.len() {
        return None;
    }
    let offset = content.floor_char_boundary(offset);
    let before = &content[..offset];
    let line = before.bytes().filter(|&b| b == b'\n').count() as u32 + 1;
    let col_start = before.rfind('\n').map_or(0, |i| i + 1);
    let col = before[col_start..].chars().count() as u32 + 1;
    Some((line, col))
}

impl Stream for TomlTomlParse {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let _ = cx;
        Poll::Ready(self.get_mut().entries.pop_front())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean() {
        assert!(lint_content(Path::new("foo.toml"), "a = 1\n[b]\nc = 2\n").is_empty());
    }

    #[test]
    fn offset_conversion() {
        assert_eq!(offset_to_line_col("a = 1\nb = 2\n", 0), Some((1, 1)));
        assert_eq!(offset_to_line_col("a = 1\nb = 2\n", 6), Some((2, 1)));
    }

    #[test]
    fn syntax_error() {
        let entries = lint_content(Path::new("foo.toml"), "a = 1\nb = [\n");
        let entry = entries.front().expect("one entry");
        assert_eq!(entry.to_string(), "foo.toml:3: [toml-parse] expected `]`");
    }

    #[test]
    fn invalid_string() {
        let entries = lint_content(Path::new("foo.toml"), "a = \"unclosed\n");
        let entry = entries.front().expect("one entry");
        assert_eq!(
            entry.to_string(),
            "foo.toml:1: [toml-parse] invalid basic string"
        );
    }
}
