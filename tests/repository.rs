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
        "Dirty.kt:2: [ktlint] Unnecessary semicolon (standard:no-semi)\n\
         clj-dirty.clj:1: [clj-kondo] unused binding x\n\
         clj-dirty.clj:2: [clj-kondo] Unresolved symbol: y\n\
         clj-dirty.clj:2: [clj-kondo] unused binding unused\n\
         docker-dirty.dockerfile:1: [hadolint] Using latest is prone to errors if the image will ever update. Pin the version explicitly to a release tag\n\
         docker-dirty.dockerfile:2: [hadolint] Delete the apt lists (/var/lib/apt/lists) after installing something\n\
         docker-dirty.dockerfile:3: [hadolint] Avoid additional packages by specifying `--no-install-recommends`\n\
         docker-dirty.dockerfile:3: [hadolint] Multiple consecutive `RUN` instructions. Consider consolidation.\n\
         docker-dirty.dockerfile:3: [hadolint] Pin versions in apt get install. Instead of `apt-get install <package>` use `apt-get install <package>=<version>`\n\
         lua-dirty.lua:1: [luacheck] unused variable 'unused'\n\
         markdown-dirty.md:4: [markdownlint-cli2] Lists should be surrounded by blank lines [Context: \"- item one\"]\n\
         markdown-dirty.md:5: [markdownlint-cli2] Trailing spaces [Expected: 0 or 2; Actual: 3]\n\
         perl-dirty.pl:1: [perlcritic] Code before strictures are enabled\n\
         python-dirty.py:1: [flake8] F401 'os' imported but unused\n\
         python-dirty.py:1: [ruff] F401 [*] `os` imported but unused\n\
         python-dirty.py:1: [ruff] I001 [*] Import block is un-sorted or un-formatted\n\
         python-dirty.py:3: [flake8] E302 expected 2 blank lines, found 1\n\
         python-dirty.py:4: [flake8] F841 local variable 'unused' is assigned to but never used\n\
         python-dirty.py:4: [ruff] F841 Local variable `unused` is assigned to but never used\n\
         python-dirty.py:7: [flake8] E305 expected 2 blank lines after class or function definition, found 1\n\
         python-dirty.py:8: [flake8] F821 undefined name 'undefined_name'\n\
         python-dirty.py:8: [ruff] F821 Undefined name `undefined_name`\n\
         shell-dirty.sh:3: [shellcheck] unused_var appears unused. Verify use (or export if used externally).\n\
         shell-dirty.sh:4: [shellcheck] Double quote to prevent globbing and word splitting.\n\
         shell-dirty.sh:5: [shellcheck] Double quote to prevent globbing and word splitting.\n\
         shell-dirty.sh:5: [shellcheck] missing_var is referenced but not assigned.\n\
         sql-dirty.sql:1: [sqlfluff] AM04: Query produces an unknown number of result columns. [ambiguous.column_count]\n\
         swift-dirty.swift:3: [swiftlint] Identifier Name Violation: Variable name 'x' should be between 3 and 40 characters long (identifier_name)\n\
         yaml-dirty.yaml:1: [yamllint] missing document start \"---\"\n\
         yaml-dirty.yaml:1: [yamllint] trailing spaces\n\
         yaml-dirty.yaml:2: [yamllint] duplication of key \"foo\" in mapping\n"
    );
}
