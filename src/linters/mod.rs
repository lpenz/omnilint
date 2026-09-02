// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use crate::cli::LinterMode;
use crate::entry::Entry;
use crate::filetype::Filetype;

use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

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
            "py_compile" => self.executable("python3"),
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
                let py_compile = py_compile::PythonPyCompile::new(self, file)?;
                let mypy = mypy::PythonMypy::new(self, file)?;
                let pyright = pyright::PythonPyright::new(self, file)?;
                Box::pin(
                    flake8
                        .merge(ruff)
                        .merge(pylint)
                        .merge(py_compile)
                        .merge(mypy)
                        .merge(pyright),
                )
            }
            Filetype::Shell => {
                let shellcheck = shellcheck::ShShellcheck::new(self, file)?;
                let bash_lint = bash::ShBash::new(self, file)?;
                let zsh_lint = zsh::ShZsh::new(self, file)?;
                Box::pin(shellcheck.merge(bash_lint).merge(zsh_lint))
            }
            Filetype::Lua => {
                if is_luau(file) {
                    Box::pin(luau::LuaLuau::new(self, file)?)
                } else {
                    let luacheck = luacheck::LuaLuacheck::new(self, file)?;
                    let luac = luac::LuaLuac::new(self, file)?;
                    let luau = luau::LuaLuau::new(self, file)?;
                    Box::pin(luacheck.merge(luac).merge(luau))
                }
            }
            Filetype::Perl => Box::pin(perlcritic::PerlPerlcritic::new(self, file)?),
            Filetype::Clojure => Box::pin(cljkondo::ClojureCljkondo::new(self, file)?),
            Filetype::Dockerfile => Box::pin(hadolint::DockerfileHadolint::new(self, file)?),
            Filetype::Kotlin => Box::pin(ktlint::KotlinKtlint::new(self, file)?),
            Filetype::Swift => Box::pin(swiftlint::SwiftSwiftlint::new(self, file)?),
            Filetype::Sql => Box::pin(sqlfluff::SqlSqlfluff::new(self, file)?),
            Filetype::Markdown => {
                let markdownlint = markdownlint::MarkdownMarkdownlint::new(self, file)?;
                let proselint = proselint::MarkdownProselint::new(self, file)?;
                Box::pin(markdownlint.merge(proselint))
            }
            Filetype::Xml => {
                let xmllint = xmllint::XmlXmllint::new(self, file)?;
                let xml_parse = xml_parse::XmlXmlParse::new(self, file)?;
                Box::pin(xmllint.merge(xml_parse))
            }
            Filetype::Html => Box::pin(tidy::HtmlTidy::new(self, file)?),
            Filetype::Json => {
                let jq = jq::JsonJq::new(self, file)?;
                let json_parse = json_parse::JsonJsonParse::new(self, file)?;
                Box::pin(jq.merge(json_parse))
            }
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
            Filetype::Systemd => Box::pin(systemd::SystemdAnalyze::new(self, file)?),
            Filetype::Javascript => {
                let oxlint = oxlint::JsOxlint::new(self, file)?;
                let eslint = eslint::JsEslint::new(self, file)?;
                Box::pin(oxlint.merge(eslint))
            }
            Filetype::Typescript => Box::pin(oxlint::JsOxlint::new(self, file)?),
            Filetype::Nix => {
                let statix = statix::NixStatix::new(self, file)?;
                let nix_compile = nix_compile::NixNixInstantiate::new(self, file)?;
                Box::pin(statix.merge(nix_compile))
            }
            Filetype::Toml => Box::pin(toml::TomlTomlParse::new(self, file)?),
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

/// Returns true if `file` is a Luau source file (`.luau` extension), which
/// should only be analysed by luau-analyze; the classic Lua tools (luacheck
/// and luac) do not understand Luau's type-annotated syntax.
fn is_luau(file: &Path) -> bool {
    file.extension().and_then(|e| e.to_str()) == Some("luau")
}

/// Parses a `filename:line:col: message` line (as emitted by flake8 and ruff)
/// into zero or one [`Entry`].
fn parse_line_standard(filename: &Path, linter: &str, line: &str) -> Vec<Entry> {
    let line = line.trim();
    let parts: Vec<&str> = line.splitn(4, ':').collect();
    let Some(line_num) = parts.get(1).and_then(|s| s.parse().ok()) else {
        return Vec::new();
    };
    let Some(col_num) = parts.get(2).and_then(|s| s.parse().ok()) else {
        return Vec::new();
    };
    let Some(msg) = parts.get(3) else {
        return Vec::new();
    };
    let msg = msg.trim();
    if line_num == 0 {
        return vec![Entry::new(filename, linter, msg).unwrap()];
    }
    vec![Entry::new_line_col(filename, linter, msg, line_num, col_num).unwrap()]
}

/// Selects how the executable for a linter is resolved.
#[derive(Clone, Copy)]
pub(crate) enum Executable {
    /// The executable is looked up by the linter's own name.
    Named,
    /// The executable is resolved through [`Linters::executable_for_linter`],
    /// for linters that run through a differently-named binary.
    Mapped,
}

/// The data that describes an external linter: how to run it and how to turn
/// each line of its output into [`Entry`] values.
#[derive(Clone, Copy)]
pub(crate) struct Spec {
    /// The linter name, used for PATH/config lookup, mode resolution and
    /// entry reporting.
    pub name: &'static str,
    /// Static arguments passed to the tool before the target filename.
    pub args: &'static [&'static str],
    /// Arguments computed at spawn time and inserted after `args` but before
    /// the target filename. Used for tools that take dynamically-derived
    /// options; most linters leave this at its default no-op.
    pub extra_args: fn() -> Vec<String>,
    /// Whether the findings are emitted on stderr rather than on stdout.
    pub findings_on_stderr: bool,
    /// How to resolve the executable to run.
    pub exec: Executable,
    /// Parses one line of the tool's output into zero or more entries.
    pub parse: fn(&Path, &str) -> Vec<Entry>,
}

impl Default for Spec {
    fn default() -> Self {
        Self {
            name: "",
            args: &[],
            extra_args: || Vec::new(),
            findings_on_stderr: false,
            exec: Executable::Named,
            parse: |_, _| Vec::new(),
        }
    }
}

/// A shared stream implementation for every linter that runs an external tool.
///
/// It spawns the tool with [`Spec`], polls its output lines, converts each
/// line into [`Entry`] values with [`Spec::parse`], and buffers the multiple
/// entries that a single line may produce. If the tool binary was not found on
/// the `PATH`, it emits a single [`Entry`] reporting that before the stream
/// ends.
pub(crate) struct CommandLinter {
    filename: PathBuf,
    inner: Linter,
    pending: VecDeque<Entry>,
    name: &'static str,
    findings_on_stderr: bool,
    parse: fn(&Path, &str) -> Vec<Entry>,
}

impl CommandLinter {
    pub(crate) fn new(
        linters: &mut Linters,
        spec: Spec,
        filename: &Path,
    ) -> color_eyre::Result<Self> {
        let executable = match spec.exec {
            Executable::Named => linters.executable(spec.name),
            Executable::Mapped => linters.executable_for_linter(spec.name),
        };
        let mut cmd = Command::new(executable.as_ref());
        for arg in spec.args {
            cmd.arg(arg);
        }
        for arg in (spec.extra_args)() {
            cmd.arg(arg);
        }
        cmd.arg(filename);
        let inner = linters.spawn(spec.name, cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
            pending: VecDeque::new(),
            name: spec.name,
            findings_on_stderr: spec.findings_on_stderr,
            parse: spec.parse,
        })
    }

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Entry>> {
        let this = self.get_mut();
        loop {
            if let Some(entry) = this.pending.pop_front() {
                return Poll::Ready(Some(entry));
            }
            match &mut this.inner {
                Linter::Running(stream) => match Pin::new(&mut **stream).poll_next(cx) {
                    Poll::Ready(Some(ProcessItem::Stdout(line))) => {
                        if !this.findings_on_stderr {
                            this.pending.extend((this.parse)(&this.filename, &line));
                        }
                    }
                    Poll::Ready(Some(ProcessItem::Stderr(line))) => {
                        if this.findings_on_stderr {
                            this.pending.extend((this.parse)(&this.filename, &line));
                        }
                    }
                    Poll::Ready(Some(_)) => continue,
                    Poll::Ready(None) => return Poll::Ready(None),
                    Poll::Pending => return Poll::Pending,
                },
                Linter::NotFound => {
                    this.inner = Linter::Done;
                    return Poll::Ready(Some(
                        Entry::new(&this.filename, this.name, "linter not found").unwrap(),
                    ));
                }
                Linter::Done => return Poll::Ready(None),
            }
        }
    }
}

/// Turns the output of a single-finding `parse_line` function into a
/// [`Spec`]-compatible `Vec<Entry>` parser.
pub(crate) fn into_entries(
    filename: &Path,
    line: &str,
    parse: fn(&Path, &str) -> Option<Entry>,
) -> Vec<Entry> {
    parse(filename, line).into_iter().collect()
}

/// Implements [`Stream`] for a wrapper struct that owns a single
/// [`CommandLinter`] in its first field, delegating the polling to it.
macro_rules! linter_stream {
    ($t:ty) => {
        impl Stream for $t {
            type Item = Entry;

            fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                Pin::new(&mut self.get_mut().0).poll_next(cx)
            }
        }
    };
}

/// Returns true if `name` is a linter built into omnilint itself, which has
/// no external executable and is therefore never "not found".
pub(crate) fn is_builtin(name: &str) -> bool {
    matches!(name, "toml-parse" | "json-parse" | "xml-parse")
}

/// All supported linter names.
pub(crate) const ALL_LINTERS: &[&str] = &[
    "actionlint",
    "bash",
    "clj-kondo",
    "cppcheck",
    "chktex",
    "eslint",
    "flake8",
    "go-vet",
    "hadolint",
    "jq",
    "json-parse",
    "ktlint",
    "luac",
    "luacheck",
    "luau-analyze",
    "markdownlint-cli2",
    "mypy",
    "nix-instantiate",
    "oxlint",
    "perlcritic",
    "protolint",
    "proselint",
    "py_compile",
    "pylint",
    "pyright",
    "ruff",
    "rubocop",
    "shellcheck",
    "sqlfluff",
    "statix",
    "staticcheck",
    "stylelint",
    "swiftlint",
    "systemd-analyze",
    "tidy",
    "toml-parse",
    "xmllint",
    "xml-parse",
    "yamllint",
    "zsh",
];

pub mod actionlint;
pub mod bash;
pub mod chktex;
pub mod cljkondo;
pub mod cppcheck;
pub mod eslint;
pub mod flake8;
pub mod govet;
pub mod hadolint;
pub mod jq;
pub mod json_parse;
pub mod ktlint;
pub mod luac;
pub mod luacheck;
pub mod luau;
pub mod markdownlint;
pub mod mypy;
pub mod nix_compile;
pub mod oxlint;
pub mod perlcritic;
pub mod proselint;
pub mod protolint;
pub mod py_compile;
pub mod pylint;
pub mod pyright;
pub mod rubocop;
pub mod ruff;
pub mod shellcheck;
pub mod sqlfluff;
pub mod staticcheck;
pub mod statix;
pub mod stylelint;
pub mod swiftlint;
pub mod systemd;
pub mod tidy;
pub mod toml;
pub mod xml_parse;
pub mod xmllint;
pub mod yamllint;
pub mod zsh;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_emits_single_entry() {
        let mut inner = CommandLinter {
            filename: PathBuf::from("foo.py"),
            inner: Linter::NotFound,
            pending: VecDeque::new(),
            name: "test",
            findings_on_stderr: false,
            parse: |_, _| Vec::new(),
        };
        let mut cx = Context::from_waker(std::task::Waker::noop());
        assert_eq!(
            Pin::new(&mut inner).poll_next(&mut cx),
            Poll::Ready(Some(
                Entry::new(Path::new("foo.py"), "test", "linter not found").unwrap()
            ))
        );
        assert_eq!(Pin::new(&mut inner).poll_next(&mut cx), Poll::Ready(None));
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
