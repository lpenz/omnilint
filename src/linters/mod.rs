// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use crate::entry::Entry;

use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll, ready};

use tokio_process_stream::{Item as ProcessItem, ProcessLineStream};
use tokio_stream::Stream;

/// Parses a `filename:line:col: message` line (as emitted by flake8 and ruff)
/// into an [`Entry`], or `None` to skip the line.
fn parse_line_standard(filename: &Path, linter: &str, line: &str) -> Option<Entry> {
    let line = line.trim();
    let parts: Vec<&str> = line.splitn(4, ':').collect();
    let line_num: u32 = parts.get(1)?.parse().ok()?;
    let col_num: u32 = parts.get(2)?.parse().ok()?;
    let msg = parts.get(3)?.trim();
    if line_num == 0 {
        return Some(Entry::new(filename, linter, msg).unwrap());
    }
    Some(Entry::new_line_col(filename, linter, msg, line_num, col_num).unwrap())
}

/// Polls a linter's `inner` process stream, converting its stdout lines into
/// [`Entry`] values via `parse` and logging any stderr output.
pub(crate) fn poll_next(
    name: &'static str,
    filename: &Path,
    inner: &mut ProcessLineStream,
    parse: fn(&Path, &str) -> Option<Entry>,
    cx: &mut Context<'_>,
) -> Poll<Option<Entry>> {
    loop {
        match ready!(Pin::new(&mut *inner).poll_next(cx)) {
            Some(ProcessItem::Stdout(line)) => {
                if let Some(entry) = parse(filename, &line) {
                    return Poll::Ready(Some(entry));
                }
            }
            Some(ProcessItem::Stderr(line)) => {
                eprintln!("[{} {}] stderr {}", name, filename.display(), line);
            }
            Some(ProcessItem::Done(_)) => {
                // Linters end in error when they find violations; the output is
                // already on stdout, so we can just ignore the exit status.
                continue;
            }
            None => return Poll::Ready(None),
        }
    }
}

pub mod flake8;
pub mod ruff;
pub mod shellcheck;
pub mod yamllint;
