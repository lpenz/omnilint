// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Built-in XML well-formedness linter.
//!
//! Unlike the external `xmllint` linter, this linter does not invoke a binary:
//! it uses the [`quick-xml`] crate to parse the file contents in-process. It
//! reports the first well-formedness error, pointing at its line and column.

use crate::cli::LinterMode;
use crate::entry::Entry;
use crate::linters::Linters;

use std::collections::VecDeque;
use std::fs;
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use quick_xml::Reader;
use quick_xml::errors::Error as XmlError;
use tokio_stream::Stream;

pub struct XmlXmlParse {
    entries: VecDeque<Entry>,
}

impl XmlXmlParse {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let entries = if linters.resolve_mode("xml-parse") == LinterMode::Disabled {
            VecDeque::new()
        } else {
            parse_file(filename)
        };
        Ok(Self { entries })
    }
}

/// Reads and parses an XML file, returning one [`Entry`] on the first
/// well-formedness error, or reporting the read error itself if the file
/// can't be read.
fn parse_file(filename: &Path) -> VecDeque<Entry> {
    let mut entries = VecDeque::new();
    let content = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(error) => {
            if let Ok(entry) = Entry::new(filename, "xml-parse", &format!("cannot read: {error}")) {
                entries.push_back(entry);
            }
            return entries;
        }
    };
    entries.extend(lint_content(filename, &content));
    entries
}

/// Consumes `content` as an XML document, returning one [`Entry`] on the
/// first well-formedness error, or none if it parses cleanly.
fn lint_content(filename: &Path, content: &str) -> VecDeque<Entry> {
    let mut entries = VecDeque::new();
    let mut reader = Reader::from_str(content);
    reader.config_mut().check_end_names = true;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Eof) => break,
            Ok(_) => buf.clear(),
            Err(error) => {
                entries.push_back(entry_for_error(filename, &reader, error));
                break;
            }
        }
    }
    entries
}

/// Converts a `quick-xml` error into an [`Entry`], using the reader's buffer
/// position to compute the line and column, or without a location when that
/// is not available.
fn entry_for_error(filename: &Path, reader: &Reader<&[u8]>, error: XmlError) -> Entry {
    let buf = reader.buffer_position();
    let msg = error.to_string();
    let pos = if msg.contains("end of") || msg.contains("Eof") {
        None
    } else {
        let content = std::str::from_utf8(reader.get_ref()).unwrap_or_default();
        offset_to_line_col(content, buf as usize)
    };
    if let Some((line, col)) = pos {
        Entry::new_line_col(filename, "xml-parse", &msg, line, col).unwrap()
    } else {
        Entry::new(filename, "xml-parse", &msg).unwrap()
    }
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

impl Stream for XmlXmlParse {
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
        assert!(lint_content(Path::new("foo.xml"), "<root><a>1</a></root>").is_empty());
    }

    #[test]
    fn mismatched_tag() {
        let entries = lint_content(Path::new("foo.xml"), "<root>\n  <a>x</b>\n</root>\n");
        let entry = entries.front().expect("one entry");
        assert_eq!(
            entry.to_string(),
            "foo.xml: [xml-parse] ill-formed document: expected `</a>`, but `</b>` was found"
        );
    }

    #[test]
    fn unclosed_tag() {
        let entries = lint_content(Path::new("foo.xml"), "<root>\n  <a>\n</root>\n");
        let entry = entries.front().expect("one entry");
        assert_eq!(
            entry.to_string(),
            "foo.xml: [xml-parse] ill-formed document: expected `</a>`, but `</root>` was found"
        );
    }

    #[test]
    fn offset_conversion() {
        assert_eq!(offset_to_line_col("<r>\n<c/>\n", 0), Some((1, 1)));
        assert_eq!(offset_to_line_col("<r>\n<c/>\n", 4), Some((2, 1)));
    }
}
