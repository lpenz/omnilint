// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use crate::entry::Entry;
use crate::filetype::Filetype;

use std::collections::HashSet;
use std::io::ErrorKind;
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll, ready};

use tokio::process::Command;
use tokio_process_stream::{Item as ProcessItem, ProcessLineStream};
use tokio_stream::{Stream, StreamExt};

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
    fn spawn(cmd: Command) -> color_eyre::Result<Self> {
        match ProcessLineStream::try_from(cmd) {
            Ok(inner) => Ok(Linter::Running(Box::new(inner))),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Linter::NotFound),
            Err(error) => Err(error.into()),
        }
    }
}

/// Manages the runtime linter instances for a run, caching which linter
/// binaries were not found on the `PATH` so that we don't keep trying to run
/// them for every file of a matching [`Filetype`].
pub(crate) struct Linters {
    not_found: HashSet<&'static str>,
}

impl Linters {
    pub(crate) fn new() -> Self {
        Self {
            not_found: HashSet::new(),
        }
    }

    /// Spawns the linter `name` with `cmd`, returning [`Linter::NotFound`]
    /// without attempting to run it again if it was already found missing.
    fn spawn(&mut self, name: &'static str, cmd: Command) -> color_eyre::Result<Linter> {
        if self.not_found.contains(name) {
            return Ok(Linter::NotFound);
        }
        match Linter::spawn(cmd) {
            Ok(Linter::NotFound) => {
                self.not_found.insert(name);
                Ok(Linter::NotFound)
            }
            result => result,
        }
    }

    /// Creates a stream that lints the given file, or `None` if there is no
    /// linter for its [`Filetype`].
    pub(crate) fn stream_for_file(
        &mut self,
        file: &Path,
    ) -> color_eyre::Result<Option<Pin<Box<dyn Stream<Item = Entry>>>>> {
        let filetype = Filetype::detect(file);
        let stream: Pin<Box<dyn Stream<Item = Entry>>> = match filetype {
            Filetype::Yaml => Box::pin(yamllint::YamlYamllint::new(self, file)?),
            Filetype::Python => {
                let flake8 = flake8::PythonFlake8::new(self, file)?;
                let ruff = ruff::PythonRuff::new(self, file)?;
                Box::pin(flake8.merge(ruff))
            }
            Filetype::Shell => Box::pin(shellcheck::ShShellcheck::new(self, file)?),
            Filetype::Lua => Box::pin(luacheck::LuaLuacheck::new(self, file)?),
            Filetype::Perl => Box::pin(perlcritic::PerlPerlcritic::new(self, file)?),
            Filetype::Clojure => Box::pin(cljkondo::ClojureCljkondo::new(self, file)?),
            Filetype::Dockerfile => Box::pin(hadolint::DockerfileHadolint::new(self, file)?),
            Filetype::Kotlin => Box::pin(ktlint::KotlinKtlint::new(self, file)?),
            Filetype::Swift => Box::pin(swiftlint::SwiftSwiftlint::new(self, file)?),
            Filetype::Sql => Box::pin(sqlfluff::SqlSqlfluff::new(self, file)?),
            _ => return Ok(None),
        };
        Ok(Some(stream))
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

/// Polls a linter's `inner` process stream, converting its lines into
/// [`Entry`] values via `parse` and logging the lines on the other stream.
/// `findings_on_stderr` selects which of the process streams holds the
/// findings; the remaining stream is logged. If the linter binary was not
/// found on the `PATH`, emits a single [`Entry`] reporting that before the
/// stream ends.
pub(crate) fn poll_next(
    name: &'static str,
    filename: &Path,
    inner: &mut Linter,
    parse: fn(&Path, &str) -> Option<Entry>,
    findings_on_stderr: bool,
    cx: &mut Context<'_>,
) -> Poll<Option<Entry>> {
    match inner {
        Linter::Running(stream) => loop {
            match ready!(Pin::new(&mut *stream).poll_next(cx)) {
                Some(ProcessItem::Stdout(line)) => {
                    if findings_on_stderr {
                        eprintln!("[{} {}] stdout {}", name, filename.display(), line);
                    } else if let Some(entry) = parse(filename, &line) {
                        return Poll::Ready(Some(entry));
                    }
                }
                Some(ProcessItem::Stderr(line)) => {
                    if findings_on_stderr {
                        if let Some(entry) = parse(filename, &line) {
                            return Poll::Ready(Some(entry));
                        }
                    } else {
                        eprintln!("[{} {}] stderr {}", name, filename.display(), line);
                    }
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

pub mod cljkondo;
pub mod flake8;
pub mod hadolint;
pub mod ktlint;
pub mod luacheck;
pub mod perlcritic;
pub mod ruff;
pub mod shellcheck;
pub mod sqlfluff;
pub mod swiftlint;
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
            poll_next("test", Path::new("foo.py"), &mut inner, parse, false, &mut cx),
            Poll::Ready(Some(
                Entry::new(Path::new("foo.py"), "test", "linter not found").unwrap()
            ))
        );
        assert_eq!(
            poll_next("test", Path::new("foo.py"), &mut inner, parse, false, &mut cx),
            Poll::Ready(None)
        );
    }

    #[test]
    fn caches_not_found() {
        let mut linters = Linters::new();
        let absent = Command::new("/nonexistent/omnilint-linter-probe");
        assert!(matches!(
            linters.spawn("probe", absent),
            Ok(Linter::NotFound)
        ));
        let present = Command::new("/bin/true");
        assert!(matches!(
            linters.spawn("probe", present),
            Ok(Linter::NotFound)
        ));
    }
}
