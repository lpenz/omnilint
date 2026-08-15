// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use crate::entry::Entry;

use std::io::ErrorKind;
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll, ready};

use tokio::process::Command;
use tokio_process_stream::{Item as ProcessItem, ProcessLineStream};
use tokio_stream::Stream;

/// The spawned process of a linter, or a placeholder for a linter binary that
/// was not found on the `PATH`.
pub(crate) enum Linter {
    Running(Box<ProcessLineStream>),
    NotFound,
    Done,
}

impl Linter {
    /// Spawns `command`, returning [`Linter::Running`] if the program was
    /// found, [`Linter::NotFound`] if it was not on the `PATH`, and an error
    /// for any other failure.
    pub(crate) fn spawn(cmd: Command) -> color_eyre::Result<Self> {
        match ProcessLineStream::try_from(cmd) {
            Ok(inner) => Ok(Linter::Running(Box::new(inner))),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Linter::NotFound),
            Err(error) => Err(error.into()),
        }
    }
}

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
/// [`Entry`] values via `parse` and logging any stderr output. If the linter
/// binary was not found on the `PATH`, emits a single [`Entry`] reporting that
/// before the stream ends.
pub(crate) fn poll_next(
    name: &'static str,
    filename: &Path,
    inner: &mut Linter,
    parse: fn(&Path, &str) -> Option<Entry>,
    cx: &mut Context<'_>,
) -> Poll<Option<Entry>> {
    match inner {
        Linter::Running(stream) => loop {
            match ready!(Pin::new(&mut *stream).poll_next(cx)) {
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
        },
        Linter::NotFound => {
            *inner = Linter::Done;
            Poll::Ready(Some(
                Entry::new(filename, name, "linter not found").unwrap(),
            ))
        }
        Linter::Done => Poll::Ready(None),
    }
}

pub mod flake8;
pub mod ruff;
pub mod shellcheck;
pub mod yamllint;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_emits_single_entry() {
        let mut inner = Linter::NotFound;
        let mut cx = Context::from_waker(std::task::Waker::noop());
        let parse = |_: &Path, _: &str| None;
        assert_eq!(
            poll_next("test", Path::new("foo.py"), &mut inner, parse, &mut cx),
            Poll::Ready(Some(
                Entry::new(Path::new("foo.py"), "test", "linter not found").unwrap()
            ))
        );
        assert_eq!(
            poll_next("test", Path::new("foo.py"), &mut inner, parse, &mut cx),
            Poll::Ready(None)
        );
    }
}
