[![CI](https://github.com/lpenz/omnilint/actions/workflows/ci.yml/badge.svg)](https://github.com/lpenz/omnilint/actions/workflows/ci.yml)
[![coveralls](https://coveralls.io/repos/github/lpenz/omnilint/badge.svg?branch=main)](https://coveralls.io/github/lpenz/omnilint?branch=main)
[![dependency status](https://deps.rs/repo/github/lpenz/omnilint/status.svg)](https://deps.rs/repo/github/lpenz/omnilint)
[![crates.io](https://img.shields.io/crates/v/omnilint)](https://crates.io/crates/omnilint)
[![packagecloud](https://img.shields.io/badge/deb-packagecloud.io-844fec.svg)](https://packagecloud.io/app/lpenz/debian/search?q=omnilint)

# omnilint

Statically analyse any file with the appropriate tools

## Table of contents

- [Features](#features)
- [Supported file types and linters](#supported-file-types-and-linters)
- [Usage](#usage)
  - [`omnilint files <files...>`](#omnilint-files-files)
  - [`omnilint repository`](#omnilint-repository)
  - [`omnilint inventory`](#omnilint-inventory)
  - [Output format](#output-format)
  - [Exit status](#exit-status)
  - [Configuration](#configuration)
- [Requirements](#requirements)
- [Installation](#installation)
  - [From crates.io](#from-cratesio)
  - [From source](#from-source)
  - [NixOS](#nixos)
  - [home-manager](#home-manager)
  - [Prebuilt packages](#prebuilt-packages)
- [GitHub Actions](#github-actions)
- [Development](#development)
- [License](#license)

## Features

- Detects the file type by extension or shebang and runs the appropriate
  linter(s) for it
- Runs all the linters in parallel
- Supports both individual files and whole repositories
- Unified output format, regardless of the linter that produced the finding

## Supported file types and linters

| File type  | Extensions / shebang          | Linters                              |
|------------|-------------------------------|--------------------------------------|
| Python     | `.py`, `#!/usr/bin/python3`, `#!/usr/bin/env python3`, ... | [flake8](https://flake8.pycqa.org/), [mypy](https://mypy-lang.org/), [py_compile](https://docs.python.org/3/library/py_compile.html), [pylint](https://pylint.readthedocs.io/), [pyright](https://microsoft.github.io/pyright/) and [ruff](https://docs.astral.sh/ruff/) |
| YAML       | `.yaml`, `.yml`               | [yamllint](https://yamllint.readthedocs.io/) and [actionlint](https://github.com/rhysd/actionlint) for GitHub Actions workflows (`.github/workflows/`) |
| Shell      | `.sh`, `.bash`, `.dash`, `.ksh`, `#!/bin/bash`, ... | [ShellCheck](https://www.shellcheck.net/) |
| Lua        | `.lua`                        | [luacheck](https://luacheck.readthedocs.io/) |
| Perl       | `.pl`, `.pm`                 | [perlcritic](https://metacpan.org/pod/Perl::Critic) |
| Clojure    | `.clj`, `.cljs`, `.cljc`, `.edn` | [clj-kondo](https://github.com/clj-kondo/clj-kondo) |
| Dockerfile | `Dockerfile`, `Dockerfile.*`, `Containerfile`, `*.dockerfile` | [hadolint](https://github.com/hadolint/hadolint) |
| Kotlin     | `.kt`, `.kts`                  | [ktlint](https://pinterest.github.io/ktlint/) |
| Swift      | `.swift`                       | [swiftlint](https://github.com/realm/SwiftLint) |
| SQL        | `.sql`                         | [sqlfluff](https://sqlfluff.com/) |
| Markdown   | `.md`, `.markdown`             | [markdownlint-cli2](https://github.com/DavidAnson/markdownlint-cli2) and [proselint](https://github.com/amperser/proselint) |
| Nix        | `.nix`                         | [nix-instantiate](https://nixos.org/manual/nix/stable/) and [statix](https://github.com/oppiliappan/statix) |
| XML        | `.xml`                         | [xmllint](https://gitlab.gnome.org/GNOME/libxml2/-/wikis/home) and a built-in [quick-xml](https://crates.io/crates/quick-xml) parser |
| HTML       | `.html`, `.htm`                | [tidy](https://www.html-tidy.org/) |
| JSON       | `.json`                        | [jq](https://jqlang.github.io/jq/) and a built-in [serde_json](https://crates.io/crates/serde_json) parser |
| C/C++      | `.c`, `.cc`, `.cpp`, `.cxx`, `.h`, `.hh`, `.hpp`, `.hxx` | [cppcheck](https://cppcheck.sourceforge.io/) |
| Protobuf   | `.proto`                       | [protolint](https://github.com/yoheimuta/protolint) |
| Go         | `.go`                          | [staticcheck](https://staticcheck.dev/) and [go vet](https://pkg.go.dev/cmd/vet) |
| Ruby       | `.rb`                          | [rubocop](https://docs.rubocop.org/) |
| CSS        | `.css`                         | [stylelint](https://stylelint.io/) |
| TeX        | `.tex`, `.sty`, `.cls`         | [chktex](https://www.nongnu.org/chktex/) |
| JavaScript | `.js`                          | [oxlint](https://oxc.rs/) and [eslint](https://eslint.org/) |
| TypeScript | `.ts`                          | [oxlint](https://oxc.rs/) |
| systemd    | `.service`, `.timer`, `.socket`, ... | [systemd-analyze verify](https://www.freedesktop.org/software/systemd/man/latest/systemd-analyze.html) |
| TOML       | `.toml`                        | built-in [toml](https://crates.io/crates/toml) parser |

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

### `omnilint inventory`

Shows the status of all supported linters, one per line, with their mode and
version when available:

```console
$ omnilint inventory
flake8               wanted     3.1.0
...
nix-instantiate      wanted     not found
toml                 wanted     built-in
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

Alternatively, `--format github-workflow` emits
[GitHub Actions workflow commands](https://docs.github.com/en/actions/reference/workflow-commands-for-github-actions)
so that findings show up directly in the pull request and commit annotations:

```text
::warning file=<filename>,line=<line>,col=<col>::[<linter>] <message>
```

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

By default, omnilint complains about a linter that is not found on the `PATH`
when a file's type requires it, which makes it exit with status `1`. The
`--ignore-missing-linters` flag makes omnilint silently skip such linters, so
they are neither reported nor counted for the exit status; it is equivalent to
setting the global [linter mode](#configuration) to `optional`. This can also
be enabled by setting the
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
`default_linter_mode`, and a `[linters.<name>]` section per linter with `mode`
and an optional `path`. When a per-linter `mode` is not set, the global
`default_linter_mode` is used. The global `ignore` option takes a list of
files to skip entirely, or directories that are skipped with all the files
inside them; it applies to both the `files` and the `repository` subcommands:

```toml
[global]
default_linter_mode = "optional"
# Skip particular files/directories, with or without glob patterns:
ignore = ["generated/tabs.out", "third_party", "**/*.pyc", "src/**"]

[linters.flake8]
mode = "disabled"

[linters.ruff]
path = "/usr/local/bin/ruff"
```

Note that `ignore` entries are relative to the directory where omnilint is
run, and list entries accumulate when config files are merged. An entry with
no glob metacharacters is a plain path: it skips that file, or, if it names a
directory, skips the directory and everything inside it. An entry with glob
metacharacters is matched as a glob pattern against the whole path; as in
`.gitignore`, `*` matches across directories, so `*.pyc` also matches `.pyc`
files in subdirectories.

The same effect can be achieved without a config file by passing the
`--default-linter-mode <mode>` flag:

```console
$ omnilint --default-linter-mode optional files test.py
```

The linter modes control how a linter that is not found on the `PATH` is
handled:

- `required`: abort with an error
- `wanted` (the default when nothing is set): emit a finding for the missing
  linter, counted for the exit status
- `optional`: run the linter if available, silently skip it otherwise
- `disabled`: never run the linter, even if the binary is available

## Requirements

The underlying linters must be installed for omnilint to analyse the
corresponding file types. How missing linters are handled depends on the
configurable [linter mode](#configuration). See the
[supported file types and linters](#supported-file-types-and-linters) table
for the complete list.

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

### NixOS

Add the flake input and use the package in your system configuration:

```nix
{
  inputs.omnilint.url = "github:lpenz/omnilint";

  outputs = { self, nixpkgs, omnilint, ... }: {
    nixosConfigurations.myhost = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        { environment.systemPackages = [ omnilint.packages.x86_64-linux.default ]; }
      ];
    };
  };
}
```

### home-manager

Add the flake input and use the package in your home configuration:

```nix
{
  inputs.omnilint.url = "github:lpenz/omnilint";

  outputs = { self, nixpkgs, omnilint, ... }: {
    homeConfigurations.myuser = {
      system = "x86_64-linux";
      modules = [
        { home.packages = [ omnilint.packages.x86_64-linux.default ]; }
      ];
    };
  };
}
```

### Prebuilt packages

- Debian/Ubuntu `.deb` packages are available on
  [packagecloud](https://packagecloud.io/app/lpenz/debian/search?q=omnilint).
- RPM packages are available on
  [packagecloud](https://packagecloud.io/app/lpenz/rpm/search?q=omnilint).
- Releases are also published on
  [GitHub](https://github.com/lpenz/omnilint/releases) with prebuilt binaries.

## GitHub Actions

omnilint can run in GitHub Actions through a reusable workflow or as a
composite action. Both use the nix flake of this repository, which provides
omnilint together with every supported linter at pinned versions, and run
`omnilint repository` with the `github-workflow` format so findings show up as
annotations on the files and lines of pull requests; the job fails whenever
omnilint finds an issue. The environment is stored in the [lpenz cachix
cache](https://lpenz.cachix.org), so repeated runs in CI and across runners
substitute the store paths instead of building them from scratch.

Using the reusable workflow, which checks out the repository for you:

```yaml
on: [push, pull_request]
jobs:
  omnilint:
    uses: lpenz/omnilint/.github/workflows/omnilint.yml@v0.9.0
    with:
      arguments: --config omnilint.toml
```

Using the action directly in a job, checking out the repository first:

```yaml
runs-on: ubuntu-latest
steps:
  - uses: actions/checkout@v7
  - uses: lpenz/omnilint@v0.9.0
    with:
      arguments: --config omnilint.toml
```

The version is pinned by the `@` reference in the `uses:` line, and can be
updated to track newer releases. Both accept an `arguments` input with extra
omnilint arguments, e.g. `--config omnilint.toml` to point at a custom
configuration file; omit `with:` entirely when no extra arguments are needed.

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
