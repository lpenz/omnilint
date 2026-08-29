[![CI](https://github.com/lpenz/omnilint/actions/workflows/ci.yml/badge.svg)](https://github.com/lpenz/omnilint/actions/workflows/ci.yml)
[![coveralls](https://coveralls.io/repos/github/lpenz/omnilint/badge.svg?branch=main)](https://coveralls.io/github/lpenz/omnilint?branch=main)
[![dependency status](https://deps.rs/repo/github/lpenz/omnilint/status.svg)](https://deps.rs/repo/github/lpenz/omnilint)
[![crates.io](https://img.shields.io/crates/v/omnilint)](https://crates.io/crates/omnilint)
[![packagecloud](https://img.shields.io/badge/deb-packagecloud.io-844fec.svg)](https://packagecloud.io/app/lpenz/debian/search?q=omnilint)

# omnilint

Statically analyse any file with the appropriate tools

## Features

- Detects the file type by extension or shebang and runs the appropriate
  linter(s) for it
- Runs all the linters in parallel
- Supports both individual files and whole repositories
- Unified output format, regardless of the linter that produced the finding

## Supported file types and linters

| File type  | Extensions / shebang          | Linters                              |
|------------|-------------------------------|--------------------------------------|
| Python     | `.py`, `#!/usr/bin/python3`, `#!/usr/bin/env python3`, ... | [flake8](https://flake8.pycqa.org/), [pylint](https://pylint.readthedocs.io/) and [ruff](https://docs.astral.sh/ruff/) |
| YAML       | `.yaml`, `.yml`               | [yamllint](https://yamllint.readthedocs.io/) and [actionlint](https://github.com/rhysd/actionlint) for GitHub Actions workflows (`.github/workflows/`) |
| Shell      | `.sh`, `.bash`, `.dash`, `.ksh`, `#!/bin/bash`, ... | [ShellCheck](https://www.shellcheck.net/) |
| Lua        | `.lua`                        | [luacheck](https://luacheck.readthedocs.io/) |
| Perl       | `.pl`, `.pm`                 | [perlcritic](https://metacpan.org/pod/Perl::Critic) |
| Clojure    | `.clj`, `.cljs`, `.cljc`, `.edn` | [clj-kondo](https://github.com/clj-kondo/clj-kondo) |
| Dockerfile | `Dockerfile`, `Dockerfile.*`, `Containerfile`, `*.dockerfile` | [hadolint](https://github.com/hadolint/hadolint) |
| Kotlin     | `.kt`, `.kts`                  | [ktlint](https://pinterest.github.io/ktlint/) |
| Swift      | `.swift`                       | [swiftlint](https://github.com/realm/SwiftLint) |
| SQL        | `.sql`                         | [sqlfluff](https://sqlfluff.com/) |
| Markdown   | `.md`, `.markdown`             | [markdownlint-cli2](https://github.com/DavidAnson/markdownlint-cli2) |
| XML        | `.xml`                         | [xmllint](https://gitlab.gnome.org/GNOME/libxml2/-/wikis/home) |
| HTML       | `.html`, `.htm`                | [tidy](https://www.html-tidy.org/) |
| JSON       | `.json`                        | [jq](https://jqlang.github.io/jq/) |
| C/C++      | `.c`, `.cc`, `.cpp`, `.cxx`, `.h`, `.hh`, `.hpp`, `.hxx` | [cppcheck](https://cppcheck.sourceforge.io/) |
| Protobuf   | `.proto`                       | [protolint](https://github.com/yoheimuta/protolint) |
| Go         | `.go`                          | [staticcheck](https://staticcheck.dev/) and [go vet](https://pkg.go.dev/cmd/vet) |
| Ruby       | `.rb`                          | [rubocop](https://docs.rubocop.org/) |
| CSS        | `.css`                         | [stylelint](https://stylelint.io/) |
| TeX        | `.tex`, `.sty`, `.cls`         | [chktex](https://www.nongnu.org/chktex/) |
| JavaScript | `.js`                          | [oxlint](https://oxc.rs/) |
| TypeScript | `.ts`                          | [oxlint](https://oxc.rs/) |

## Usage

### `omnilint files <files...>`

Analyses the given files with the appropriate tools:

```console
$ omnilint files test.py
test.py:1: [ruff] F401 'os' imported but unused
test.py:12: [flake8] E501 line too long (95 > 79 characters)
```

### `omnilint repository`

Analyses all the files tracked by git in the current repository:

```console
$ omnilint repository
src/main.rs:5: [shellcheck] SC2148: Tips depend on target shell and yours is unknown.
```

### Output format

Findings are printed to stderr in the format:

```text
<filename>:<line>: [<linter>] <message>
```

When a linter reports a file-level issue with no line number, the `line` part
is omitted:

```text
<filename>: [<linter>] <message>
```

This format is similar to the one used by compilers, and is parseable by most
editors and IDEs.

### Exit status

omnilint exits with status `0` when no issues were found, and with status `1`
when at least one finding was emitted, including when a linter was not found
on the `PATH`. This makes it usable as a gate in CI pipelines and git hooks:

```console
$ omnilint files test.py && echo "clean"
test.py:1: [ruff] F401 'os' imported but unused
$ echo $?
1
```

The `--ignore-missing-linters` flag makes omnilint silently skip linters that
are not found on the `PATH`, so they are neither reported nor counted for the
exit status. This can also be enabled by setting the
`OMNILINT_IGNORE_MISSING_LINTERS` environment variable to a truthy value
(`1`, `true`, `yes` or `on`):

```console
$ omnilint --ignore-missing-linters files test.py
$ echo $?
0
$ OMNILINT_IGNORE_MISSING_LINTERS=1 omnilint files test.py
$ echo $?
0
```

### Configuration

omnilint is configured through one or more TOML files. The `--config <path>`
option makes omnilint use a specific configuration file instead of the
automatic discovery, which is useful to point at a custom config in CI or
scripts:

```console
$ omnilint --config /path/to/omnilint.toml files test.py
```

When `--config` is not given, omnilint loads and merges configuration from the
following sources, in order of increasing precedence:

1. the `OMNILINT_CONFIG` environment variable pointing to a file
2. `/etc/omnilint.toml`
3. `~/.config/omnilint/omnilint.toml`
4. `./omnilint.toml` in the current directory

A config file has a `[global]` section for global options such as
`default_linter_mode` (one of `required`, `wanted`, `optional` or `disabled`),
and a `[linters.<name>]` section per linter with `mode` and an optional `path`.
When a per-linter `mode` is not set, the global `default_linter_mode` is used:

```toml
[global]
default_linter_mode = "optional"

[linters.flake8]
mode = "disabled"

[linters.ruff]
path = "/usr/local/bin/ruff"
```

## Requirements

The underlying linters must be installed for omnilint to analyse the
corresponding file types. When a linter is not found on the `PATH`, omnilint
does not abort; instead, it emits an entry saying that the linter was not
found:

```text
<filename>: [<linter>] linter not found
```

The linters used are:

- `flake8`, `pylint` and `ruff` for Python
- `yamllint` for YAML, and `actionlint` for GitHub Actions workflow files
  (under `.github/workflows/`)
- `shellcheck` for Shell
- `luacheck` for Lua
- `perlcritic` for Perl
- `clj-kondo` for Clojure
- `hadolint` for Dockerfile
- `ktlint` for Kotlin
- `swiftlint` for Swift
- `sqlfluff` for SQL
- `markdownlint-cli2` for Markdown
- `xmllint` for XML
- `tidy` for HTML
- `jq` for JSON
- `cppcheck` for C/C++
- `protolint` for Protobuf
- `staticcheck` and `go vet` for Go
- `rubocop` for Ruby
- `stylelint` for CSS
- `chktex` for TeX/LaTeX
- `oxlint` for JavaScript and TypeScript

## Installation

### From crates.io

```console
$ cargo install omnilint
```

### From source

```console
$ git clone https://github.com/lpenz/omnilint
$ cd omnilint
$ cargo install --path .
```

### Prebuilt packages

- Debian/Ubuntu `.deb` packages are available on
  [packagecloud](https://packagecloud.io/app/lpenz/debian/search?q=omnilint).
- RPM packages are available on
  [packagecloud](https://packagecloud.io/app/lpenz/rpm/search?q=omnilint).
- Releases are also published on
  [GitHub](https://github.com/lpenz/omnilint/releases) with prebuilt binaries.

## Development

Use the provided [nix](https://nixos.org/) flake to get a development shell
with all the linter tools installed:

```console
$ nix develop
```

Run the test suite:

```console
$ cargo test
$ cargo test --features test-linter-tools   # also requires the linter tools
```

## License

omnilint is licensed under the MIT license. See the
[LICENSE](LICENSE) file for details.
