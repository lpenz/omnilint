// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the `repository` subcommand, which lints all the
//! files tracked by git in the current repository.
//!
//! Requires the linter tools to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn all_tracked_files() {
    assert_eq!(
        common::run_repository(),
        ".github/workflows/clean.yml:3: [yamllint] truthy value should be one of [false, true]\n\
         .github/workflows/dirty.yml:3: [yamllint] truthy value should be one of [false, true]\n\
         .github/workflows/dirty.yml:8: [actionlint] element of \"steps\" section is scalar node but mapping node is expected\n\
         .github/workflows/dirty.yml:8: [actionlint] step must run script with \"run\" section or run action with \"uses\" section\n\
         .rubocop.yml:1: [yamllint] missing document start \"---\"\n\
         Dirty.kt:2: [ktlint] Unnecessary semicolon (standard:no-semi)\n\
         Error: lint findings were emitted\n\
         bash-dirty.sh: [bash] line 4: syntax error: unexpected end of file from `if' command on line 1\n\
         bash-dirty.sh:1: [shellcheck] Couldn't find 'fi' for this 'if'.\n\
         bash-dirty.sh:1: [shellcheck] Couldn't parse this if expression. Fix to allow more checks.\n\
         bash-dirty.sh:4: [shellcheck] Expected 'fi' matching previously mentioned 'if'.\n\
         bash-dirty.sh:4: [shellcheck] Expected 'fi'. Fix any mentioned problems and try again.\n\
         bash-dirty.sh:4: [zsh] parse error near `\\n'\n\
         c-dirty.c:5: [cppcheck] Memory leak: p [memleak]\n\
         clj-dirty.clj:1: [clj-kondo] unused binding x\n\
         clj-dirty.clj:2: [clj-kondo] Unresolved symbol: y\n\
         clj-dirty.clj:2: [clj-kondo] unused binding unused\n\
         css-dirty.css:1: [stylelint] Empty block (block-no-empty) [error]\n\
         docker-dirty.dockerfile:1: [hadolint] Using latest is prone to errors if the image will ever update. Pin the version explicitly to a release tag\n\
         docker-dirty.dockerfile:2: [hadolint] Delete the apt lists (/var/lib/apt/lists) after installing something\n\
         docker-dirty.dockerfile:3: [hadolint] Avoid additional packages by specifying `--no-install-recommends`\n\
         docker-dirty.dockerfile:3: [hadolint] Multiple consecutive `RUN` instructions. Consider consolidation.\n\
         docker-dirty.dockerfile:3: [hadolint] Pin versions in apt get install. Instead of `apt-get install <package>` use `apt-get install <package>=<version>`\n\
         go-dirty.go:5: [staticcheck] func unused is unused (U1000)\n\
         go-dirty.go:6: [go-vet] fmt.Printf format %s reads arg #1, but call has 0 args\n\
         go-dirty.go:6: [staticcheck] Printf format %s reads arg #1, but call has only 0 args (SA5009)\n\
         html-dirty.html:1: [tidy] missing <!DOCTYPE> declaration\n\
         js-dirty.js:1: [eslint] Parsing error: Unexpected keyword 'debugger'\n\
         js-dirty.js:1: [oxlint] Identifier expected. 'debugger' is a reserved word that cannot be used here. [Error]\n\
         json-dirty.json:1: [jq] Unmatched '}'\n\
         json-dirty.json:1: [json-parse] expected value at line 1 column 15\n\
         lua-dirty.lua:1: [luacheck] unused variable 'unused'\n\
         lua-dirty.lua:1: [luau-analyze] Variable 'unused' is never used; prefix with '_' to silence\n\
         luau-dirty.luau:2: [luau-analyze] Expected this to be 'number', but got 'string'\n\
         markdown-dirty.md:4: [markdownlint-cli2] Lists should be surrounded by blank lines [Context: \"- item one\"]\n\
         markdown-dirty.md:5: [markdownlint-cli2] Trailing spaces [Expected: 0 or 2; Actual: 3]\n\
         markdown-prose-dirty.md:3: [proselint] uncomparables: Comparison of an uncomparable: 'very unique' is not comparable.\n\
         markdown-prose-dirty.md:3: [proselint] weasel_words.very: Substitute 'damn' every time you're inclined to write 'very'; your editor will delete it and the writing will be just as it should be.\n\
         markdown-prose-dirty.md:5: [proselint] misc.greylist: Use of 'utilize'. Do you know anyone who needs to utilize the word utilize?\n\
         markdown-prose-dirty.md:6: [proselint] redundancy.misc.after_the_deadline: Redundancy. Use 'every' instead of 'each and every'.\n\
         nix-compile-dirty.nix:1: [nix-instantiate] syntax error\n\
         nix-compile-dirty.nix:1: [statix] Unexpected TOKEN_R_BRACE at 18..19, wanted any of [TOKEN_SEMICOLON]\n\
         nix-compile-dirty.nix:1: [statix] Unexpected end of file\n\
         nix-compile-dirty.nix:1: [statix] Unexpected end of file\n\
         nix-compile-dirty.nix:1: [statix] Unexpected end of file\n\
         nix-compile-dirty.nix:1: [statix] Unexpected end of file, wanted any of [TOKEN_SEMICOLON]\n\
         nix-compile-dirty.nix:1: [statix] Unexpected end of file, wanted any of [TOKEN_SEMICOLON]\n\
         nix-dirty.nix:2: [statix] Consider quoting this URI expression\n\
         nix-dirty.nix:3: [statix] Useless parentheses around primitive expression\n\
         perl-dirty.pl:1: [perlcritic] Code before strictures are enabled\n\
         proto_dirty.proto:4: [protolint] Found an incorrect indentation style \"\t\". \"  \" is correct.\n\
         python-clean.py:1: [pylint] Constant name \"x\" doesn't conform to UPPER_CASE naming style (invalid-name)\n\
         python-clean.py:1: [pylint] Missing module docstring (missing-module-docstring)\n\
         python-clean.py:1: [pylint] Module name \"python-clean\" doesn't conform to snake_case naming style (invalid-name)\n\
         python-dirty.py:1: [flake8] F401 'os' imported but unused\n\
         python-dirty.py:1: [pylint] Missing module docstring (missing-module-docstring)\n\
         python-dirty.py:1: [pylint] Module name \"python-dirty\" doesn't conform to snake_case naming style (invalid-name)\n\
         python-dirty.py:1: [pylint] Unused import os (unused-import)\n\
         python-dirty.py:1: [ruff] F401 [*] `os` imported but unused\n\
         python-dirty.py:1: [ruff] I001 [*] Import block is un-sorted or un-formatted\n\
         python-dirty.py:3: [flake8] E302 expected 2 blank lines, found 1\n\
         python-dirty.py:3: [pylint] Missing function or method docstring (missing-function-docstring)\n\
         python-dirty.py:4: [flake8] F841 local variable 'unused' is assigned to but never used\n\
         python-dirty.py:4: [pylint] Unused variable 'unused' (unused-variable)\n\
         python-dirty.py:4: [ruff] F841 Local variable `unused` is assigned to but never used\n\
         python-dirty.py:7: [flake8] E305 expected 2 blank lines after class or function definition, found 1\n\
         python-dirty.py:7: [pylint] Assigning result of a function call, where the function has no return (assignment-from-no-return)\n\
         python-dirty.py:7: [pylint] Constant name \"x\" doesn't conform to UPPER_CASE naming style (invalid-name)\n\
         python-dirty.py:8: [flake8] F821 undefined name 'undefined_name'\n\
         python-dirty.py:8: [mypy] Name \"undefined_name\" is not defined (name-defined)\n\
         python-dirty.py:8: [pylint] Undefined variable 'undefined_name' (undefined-variable)\n\
         python-dirty.py:8: [pyright] \"undefined_name\" is not defined (reportUndefinedVariable)\n\
         python-dirty.py:8: [ruff] F821 Undefined name `undefined_name`\n\
         ruby-dirty.rb:1: [rubocop] C: [Correctable] Style/FrozenStringLiteralComment: Missing frozen string literal comment.\n\
         shell-dirty.sh:3: [shellcheck] unused_var appears unused. Verify use (or export if used externally).\n\
         shell-dirty.sh:4: [shellcheck] Double quote to prevent globbing and word splitting.\n\
         shell-dirty.sh:5: [shellcheck] Double quote to prevent globbing and word splitting.\n\
         shell-dirty.sh:5: [shellcheck] missing_var is referenced but not assigned.\n\
         sql-dirty.sql:1: [sqlfluff] AM04: Query produces an unknown number of result columns. [ambiguous.column_count]\n\
         swift-dirty.swift:3: [swiftlint] Identifier Name Violation: Variable name 'x' should be between 3 and 40 characters long (identifier_name)\n\
         systemd-dirty.service:5: [systemd-analyze] Unknown key 'Foo' in section [Service], ignoring.\n\
         tex-dirty.tex:3: [chktex] Use ' to end quotation, not `.\n\
         toml-dirty.toml:6: [toml-parse] duplicate key `server` in document root\n\
         ts-dirty.ts:1: [oxlint] Identifier expected. 'debugger' is a reserved word that cannot be used here. [Error]\n\
         xml-dirty.xml:3: [xmllint] expected '>'\n\
         yaml-dirty.yaml:1: [yamllint] missing document start \"---\"\n\
         yaml-dirty.yaml:1: [yamllint] trailing spaces\n\
         yaml-dirty.yaml:2: [yamllint] duplication of key \"foo\" in mapping\n\
         zsh-clean.zsh:1: [shellcheck] ShellCheck only supports sh/bash/dash/ksh/'busybox sh' scripts. Sorry!\n\
         zsh-dirty.zsh:1: [shellcheck] ShellCheck only supports sh/bash/dash/ksh/'busybox sh' scripts. Sorry!\n"
    );
}
