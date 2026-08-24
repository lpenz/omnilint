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
| Python     | `.py`, `#!/usr/bin/python3`, `#!/usr/bin/env python3`, ... | [flake8](https://flake8.pycqa.org/), [mypy](https://mypy-lang.org/), [pylint](https://pylint.readthedocs.io/), [pyright](https://microsoft.github.io/pyright/) and [ruff](https://docs.astral.sh/ruff/) |
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
| Nix        | `.nix`                         | [statix](https://github.com/oppiliappan/statix) |
| XML        | `.xml`                         | [xmllint](https://gitlab.gnome.org/GNOME/libxml2/-/wikis/home) |
| HTML       | `.html`, `.htm`                | [tidy](https://www.html-tidy.org/) |
| JSON       | `.json`                        | [jq](https://jqlang.github.io/jq/) |
| C/C++      | `.c`, `.cc`, `.cpp`, `.cxx`, `.h`, `.hh`, `.hpp`, `.hxx` | [cppcheck](https://cppcheck.sourceforge.io/) |
| Protobuf   | `.proto`                       | [protolint](https://github.com/yoheimuta/protolint) |
| Go         | `.go`                          | [staticcheck](https://staticcheck.dev/) and [go vet](https://pkg.go.dev/cmd/vet) |
| Haskell    | `.hs`                          | [hlint](https://github.com/ndmitchell/hlint) |
| Ruby       | `.rb`                          | [rubocop](https://docs.rubocop.org/) and [standardrb](https://github.com/standardrb/standard) |
| CSS        | `.css`                         | [stylelint](https://stylelint.io/) |
| TeX        | `.tex`, `.sty`, `.cls`         | [chktex](https://www.nongnu.org/chktex/) |
| Terraform  | `.tf`                          | [tflint](https://github.com/terraform-linters/tflint) |
| JavaScript | `.js`                          | [oxlint](https://oxc.rs/) and [eslint](https://eslint.org/) |
| TypeScript | `.ts`                          | [oxlint](https://oxc.rs/) |
| PHP        | `.php`                         | [php -l](https://www.php.net/), [phpcs](https://github.com/PHPCSStandards/PHP_CodeSniffer) and [phpmd](https://phpmd.org/) |
| R          | `.R`, `.r`                     | [lintr](https://lintr.r-lib.org/) |
| Text       | `.txt`                         | [proselint](https://github.com/amperser/proselint) |
| SaltStack  | `.sls`                         | [salt-lint](https://github.com/warpnet/salt-lint) |
| Bazel      | `.bzl`, `BUILD`, `WORKSPACE`, ... | [buildifier](https://github.com/bazelbuild/buildtools) |
| systemd    | `.service`, `.timer`, `.socket`, ... | [systemd-analyze verify](https://www.freedesktop.org/software/systemd/man/latest/systemd-analyze.html) |
| Verilog    | `.v`, `.sv`, `.vh`, `.svh`     | [verilator --lint-only](https://www.veripool.org/verilator/) |

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

## Requirements

The underlying linters must be installed for omnilint to analyse the
corresponding file types. When a linter is not found on the `PATH`, omnilint
does not abort; instead, it emits an entry saying that the linter was not
found:

```text
<filename>: [<linter>] linter not found
```

The linters used are:

- `flake8`, `mypy`, `pylint`, `pyright` and `ruff` for Python
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
- `statix` for Nix
- `xmllint` for XML
- `tidy` for HTML
- `jq` for JSON
- `cppcheck` for C/C++
- `protolint` for Protobuf
- `staticcheck` and `go vet` for Go
- `hlint` for Haskell
- `rubocop` and `standardrb` for Ruby
- `stylelint` for CSS
- `chktex` for TeX/LaTeX
- `tflint` for Terraform
- `oxlint` and `eslint` for JavaScript, `oxlint` for TypeScript. eslint only
  reports findings when an [eslint configuration file](https://eslint.org/docs/latest/use/configure/)
  is present in the directory tree
- `php` (with `-l`), `phpcs` and `phpmd` for PHP
- `Rscript` with the `lintr` package for R
- `proselint` for plain text files
- `salt-lint` for SaltStack state files
- `buildifier` for Bazel `BUILD`, `WORKSPACE` and `.bzl` files
- `systemd-analyze verify` for systemd unit files
- `verilator --lint-only` for Verilog and SystemVerilog files

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
