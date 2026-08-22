// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use crate::cli::LinterMode;
use crate::entry::Entry;
use crate::filetype::Filetype;

use std::borrow::Cow;
use std::collections::HashMap;
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
#[derive(Debug)]
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
    default_mode: LinterMode,
    mode_overrides: HashMap<String, LinterMode>,
    executables: HashMap<String, String>,
}

impl Linters {
    pub(crate) fn new() -> Self {
        Self {
            not_found: HashSet::new(),
            default_mode: LinterMode::Wanted,
            mode_overrides: HashMap::new(),
            executables: HashMap::new(),
        }
    }

    /// Sets the default [`LinterMode`] for linters that are not found.
    pub(crate) fn set_default_mode(&mut self, mode: LinterMode) {
        self.default_mode = mode;
    }

    /// Sets per-linter [`LinterMode`] overrides.
    pub(crate) fn set_mode_overrides(&mut self, overrides: HashMap<String, LinterMode>) {
        self.mode_overrides = overrides;
    }

    /// Sets custom executable paths for linters.
    pub(crate) fn set_executables(&mut self, executables: HashMap<String, String>) {
        self.executables = executables;
    }

    /// Returns the effective executable for `name`: the custom path if one
    /// is configured, or `name` itself for PATH lookup.
    pub(crate) fn executable<'a>(&'a self, name: &'a str) -> Cow<'a, str> {
        self.executables
            .get(name)
            .map(|s| Cow::Borrowed(s.as_str()))
            .unwrap_or(Cow::Borrowed(name))
    }

    /// Returns the effective executable for linter `name`, mapping linter
    /// names that are run through a different binary to the one used for
    /// the `PATH`/config lookup: e.g. the `go-vet` linter is executed as
    /// `go`.
    pub(crate) fn executable_for_linter<'a>(&'a self, name: &'a str) -> Cow<'a, str> {
        match name {
            "go-vet" => self.executable("go"),
            _ => self.executable(name),
        }
    }

    /// Returns the effective [`LinterMode`] for `name`: per-linter override
    /// if set, otherwise the global default.
    pub(crate) fn resolve_mode(&self, name: &str) -> LinterMode {
        self.mode_overrides
            .get(name)
            .copied()
            .unwrap_or(self.default_mode)
    }

    /// Returns the [`Linter`] placeholder for a missing linter, or
    /// [`Linter::Done`] to silently skip it when the mode is
    /// [`LinterMode::Optional`].
    fn missing(&mut self, name: &'static str, mode: LinterMode) -> color_eyre::Result<Linter> {
        self.not_found.insert(name);
        match mode {
            LinterMode::Required => Err(color_eyre::eyre::eyre!(
                "required linter '{}' not found",
                name
            )),
            LinterMode::Wanted => Ok(Linter::NotFound),
            LinterMode::Optional => Ok(Linter::Done),
            LinterMode::Disabled => unreachable!(),
        }
    }

    /// Spawns the linter `name` with `cmd`, returning [`Linter::NotFound`]
    /// without attempting to run it again if it was already found missing.
    fn spawn(&mut self, name: &'static str, cmd: Command) -> color_eyre::Result<Linter> {
        let mode = self.resolve_mode(name);
        if mode == LinterMode::Disabled {
            return Ok(Linter::Done);
        }
        if self.not_found.contains(name) {
            return self.missing(name, mode);
        }
        match Linter::spawn(cmd) {
            Ok(Linter::NotFound) => self.missing(name, mode),
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
            Filetype::Yaml => {
                let yamllint = yamllint::YamlYamllint::new(self, file)?;
                if is_github_workflow(file) {
                    let actionlint = actionlint::GithubWorkflowActionlint::new(self, file)?;
                    Box::pin(yamllint.merge(actionlint))
                } else {
                    Box::pin(yamllint)
                }
            }
            Filetype::Python => {
                let flake8 = flake8::PythonFlake8::new(self, file)?;
                let ruff = ruff::PythonRuff::new(self, file)?;
                let pylint = pylint::PythonPylint::new(self, file)?;
                Box::pin(flake8.merge(ruff).merge(pylint))
            }
            Filetype::Shell => Box::pin(shellcheck::ShShellcheck::new(self, file)?),
            Filetype::Lua => Box::pin(luacheck::LuaLuacheck::new(self, file)?),
            Filetype::Perl => Box::pin(perlcritic::PerlPerlcritic::new(self, file)?),
            Filetype::Clojure => Box::pin(cljkondo::ClojureCljkondo::new(self, file)?),
            Filetype::Dockerfile => Box::pin(hadolint::DockerfileHadolint::new(self, file)?),
            Filetype::Kotlin => Box::pin(ktlint::KotlinKtlint::new(self, file)?),
            Filetype::Swift => Box::pin(swiftlint::SwiftSwiftlint::new(self, file)?),
            Filetype::Sql => Box::pin(sqlfluff::SqlSqlfluff::new(self, file)?),
            Filetype::Markdown => Box::pin(markdownlint::MarkdownMarkdownlint::new(self, file)?),
            Filetype::Xml => Box::pin(xmllint::XmlXmllint::new(self, file)?),
            Filetype::Html => Box::pin(tidy::HtmlTidy::new(self, file)?),
            Filetype::Json => Box::pin(jq::JsonJq::new(self, file)?),
            Filetype::C => Box::pin(cppcheck::CCppcheck::new(self, file)?),
            Filetype::Proto => Box::pin(protolint::ProtoProtolint::new(self, file)?),
            Filetype::Go => {
                let staticcheck = staticcheck::GoStaticcheck::new(self, file)?;
                let govet = govet::GoGovet::new(self, file)?;
                Box::pin(staticcheck.merge(govet))
            }
            Filetype::Ruby => Box::pin(rubocop::RubyRubocop::new(self, file)?),
            Filetype::Css => Box::pin(stylelint::CssStylelint::new(self, file)?),
            Filetype::TeX => Box::pin(chktex::TeXChktex::new(self, file)?),
            Filetype::Javascript => {
                let oxlint = oxlint::JsOxlint::new(self, file)?;
                let eslint = eslint::JsEslint::new(self, file)?;
                Box::pin(oxlint.merge(eslint))
            }
            Filetype::Typescript => Box::pin(oxlint::JsOxlint::new(self, file)?),
            _ => return Ok(None),
        };
        Ok(Some(stream))
    }
}

/// Returns true if `file` is a GitHub Actions workflow, i.e. it has a
/// `.github` directory component followed by a `workflows` directory
/// component.
fn is_github_workflow(file: &Path) -> bool {
    file.components()
        .zip(file.components().skip(1))
        .any(|(a, b)| a.as_os_str() == ".github" && b.as_os_str() == "workflows")
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
/// [`Entry`] values via `parse` and discarding the lines on the other stream.
/// `findings_on_stderr` selects which of the process streams holds the
/// findings. If the linter binary was not found on the `PATH`, emits a single
/// [`Entry`] reporting that before the stream ends.
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
                    if !findings_on_stderr && let Some(entry) = parse(filename, &line) {
                        return Poll::Ready(Some(entry));
                    }
                }
                Some(ProcessItem::Stderr(line)) => {
                    if findings_on_stderr && let Some(entry) = parse(filename, &line) {
                        return Poll::Ready(Some(entry));
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

/// All supported linter names.
pub(crate) const ALL_LINTERS: &[&str] = &[
    "actionlint",
    "clj-kondo",
    "cppcheck",
    "chktex",
    "eslint",
    "flake8",
    "go-vet",
    "hadolint",
    "jq",
    "ktlint",
    "luacheck",
    "markdownlint-cli2",
    "oxlint",
    "perlcritic",
    "protolint",
    "pylint",
    "ruff",
    "rubocop",
    "shellcheck",
    "sqlfluff",
    "staticcheck",
    "stylelint",
    "swiftlint",
    "tidy",
    "xmllint",
    "yamllint",
];

pub mod actionlint;
pub mod chktex;
pub mod cljkondo;
pub mod cppcheck;
pub mod eslint;
pub mod flake8;
pub mod govet;
pub mod hadolint;
pub mod jq;
pub mod ktlint;
pub mod luacheck;
pub mod markdownlint;
pub mod oxlint;
pub mod perlcritic;
pub mod protolint;
pub mod pylint;
pub mod rubocop;
pub mod ruff;
pub mod shellcheck;
pub mod sqlfluff;
pub mod staticcheck;
pub mod stylelint;
pub mod swiftlint;
pub mod tidy;
pub mod xmllint;
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
            poll_next(
                "test",
                Path::new("foo.py"),
                &mut inner,
                parse,
                false,
                &mut cx
            ),
            Poll::Ready(Some(
                Entry::new(Path::new("foo.py"), "test", "linter not found").unwrap()
            ))
        );
        assert_eq!(
            poll_next(
                "test",
                Path::new("foo.py"),
                &mut inner,
                parse,
                false,
                &mut cx
            ),
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

    #[test]
    fn disabled_skips_entirely() {
        let mut linters = Linters::new();
        linters.set_mode_overrides(
            [("probe".to_string(), LinterMode::Disabled)]
                .into_iter()
                .collect(),
        );
        let present = Command::new("/bin/true");
        assert!(matches!(linters.spawn("probe", present), Ok(Linter::Done)));
    }

    #[test]
    fn optional_skips_when_not_found() {
        let mut linters = Linters::new();
        linters.set_mode_overrides(
            [("probe".to_string(), LinterMode::Optional)]
                .into_iter()
                .collect(),
        );
        let absent = Command::new("/nonexistent/omnilint-linter-probe");
        assert!(matches!(linters.spawn("probe", absent), Ok(Linter::Done)));
    }

    #[test]
    fn required_aborts_when_not_found() {
        let mut linters = Linters::new();
        linters.set_mode_overrides(
            [("probe".to_string(), LinterMode::Required)]
                .into_iter()
                .collect(),
        );
        let absent = Command::new("/nonexistent/omnilint-linter-probe");
        let result = linters.spawn("probe", absent);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("required linter 'probe' not found")
        );
    }

    #[test]
    fn resolve_mode_prefers_override() {
        let mut linters = Linters::new();
        linters.set_default_mode(LinterMode::Required);
        linters.set_mode_overrides(
            [
                ("configured".to_string(), LinterMode::Optional),
                ("explicit".to_string(), LinterMode::Disabled),
            ]
            .into_iter()
            .collect(),
        );
        assert_eq!(linters.resolve_mode("configured"), LinterMode::Optional);
        assert_eq!(linters.resolve_mode("explicit"), LinterMode::Disabled);
        assert_eq!(linters.resolve_mode("unconfigured"), LinterMode::Required);
    }

    #[test]
    fn github_workflow_detection() {
        assert!(is_github_workflow(Path::new(".github/workflows/ci.yml")));
        assert!(is_github_workflow(Path::new(
            "foo/.github/workflows/ci.yml"
        )));
        assert!(!is_github_workflow(Path::new(".github/ci.yml")));
        assert!(!is_github_workflow(Path::new("workflows/ci.yml")));
        assert!(!is_github_workflow(Path::new("foo.yml")));
    }
}
