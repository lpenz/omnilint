// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Built-in JSON parser linter.
//!
//! Unlike the external `jq` linter, this linter does not invoke a binary: it
//! uses the [`serde_json`] crate to parse the file contents in-process and
//! report strict syntax errors. While `jq` accepts some non-standard input,
//! this linter rejects anything that is not valid JSON, and points at the
//! offending line and column.

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

pub struct JsonJsonParse {
    entries: VecDeque<Entry>,
}

impl JsonJsonParse {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let entries = if linters.resolve_mode("json-parse") == LinterMode::Disabled {
            VecDeque::new()
        } else {
            parse_file(filename)
        };
        Ok(Self { entries })
    }
}

/// Parses a JSON file, returning one [`Entry`] on the first syntax error, or
/// reporting the read error itself if the file can't be read.
fn parse_file(filename: &Path) -> VecDeque<Entry> {
    let mut entries = VecDeque::new();
    let content = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(error) => {
            if let Ok(entry) = Entry::new(filename, "json-parse", &format!("cannot read: {error}"))
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
/// the content is a valid JSON document.
fn lint_content(filename: &Path, content: &str) -> VecDeque<Entry> {
    let mut entries = VecDeque::new();
    if let Err(error) = serde_json::from_str::<serde_json::Value>(content) {
        let msg = error.to_string();
        let line = serde_json::Error::line(&error) as u32;
        let col = serde_json::Error::column(&error) as u32;
        if let Ok(entry) = if line > 0 {
            Entry::new_line_col(filename, "json-parse", &msg, line, col)
        } else {
            Entry::new(filename, "json-parse", &msg)
        } {
            entries.push_back(entry);
        }
    }
    entries
}

impl Stream for JsonJsonParse {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(self.get_mut().entries.pop_front())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean() {
        assert!(lint_content(Path::new("foo.json"), r#"{"a": [1, 2, 3]}"#).is_empty());
    }

    #[test]
    fn syntax_error() {
        let entries = lint_content(Path::new("foo.json"), "{\n\"a\": 1,\n\"b\": [\n}\n");
        let entry = entries.front().expect("one entry");
        assert_eq!(
            entry.to_string(),
            "foo.json:4: [json-parse] expected value at line 4 column 1"
        );
    }
}
