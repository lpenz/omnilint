// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the behavior when the linter tools are not found on
//! the `PATH`: instead of aborting, an entry reporting the missing linter is
//! produced for each file.

mod common;

#[test]
fn linters_not_found() {
    assert_eq!(
        common::run_without_linters(&[
            ".github/workflows/clean.yml",
            "Clean.kt",
            "c-clean.c",
            "clj-clean.clj",
            "docker-clean.dockerfile",
            "html-clean.html",
            "json-clean.json",
            "lua-clean.lua",
            "markdown-clean.md",
            "nix-clean.nix",
            "proto_clean.proto",
            "python-clean.py",
            "shell-clean.sh",
            "sql-clean.sql",
            "swift-clean.swift",
            "systemd-clean.service",
            "xml-clean.xml",
            "yaml-clean.yaml",
        ]),
        ".github/workflows/clean.yml: [actionlint] linter not found\n\
         .github/workflows/clean.yml: [yamllint] linter not found\n\
         Clean.kt: [ktlint] linter not found\n\
         Error: lint findings were emitted\n\
         c-clean.c: [cppcheck] linter not found\n\
         clj-clean.clj: [clj-kondo] linter not found\n\
         docker-clean.dockerfile: [hadolint] linter not found\n\
         html-clean.html: [tidy] linter not found\n\
         json-clean.json: [jq] linter not found\n\
         lua-clean.lua: [luac] linter not found\n\
         lua-clean.lua: [luacheck] linter not found\n\
         lua-clean.lua: [luau-analyze] linter not found\n\
         markdown-clean.md: [markdownlint-cli2] linter not found\n\
         nix-clean.nix: [nix-instantiate] linter not found\n\
         nix-clean.nix: [statix] linter not found\n\
         proto_clean.proto: [protolint] linter not found\n\
         python-clean.py: [flake8] linter not found\n\
         python-clean.py: [mypy] linter not found\n\
         python-clean.py: [py_compile] linter not found\n\
         python-clean.py: [pylint] linter not found\n\
         python-clean.py: [pyright] linter not found\n\
         python-clean.py: [ruff] linter not found\n\
         shell-clean.sh: [shellcheck] linter not found\n\
         sql-clean.sql: [sqlfluff] linter not found\n\
         swift-clean.swift: [swiftlint] linter not found\n\
         systemd-clean.service: [systemd-analyze] linter not found\n\
         xml-clean.xml: [xmllint] linter not found\n\
         yaml-clean.yaml: [yamllint] linter not found\n"
    );
}

#[test]
fn ignore_missing_linters() {
    assert_eq!(
        common::run_ignore_missing_linters(&["python-clean.py", "yaml-clean.yaml"]),
        ""
    );
}

#[test]
fn ignore_missing_linters_config() {
    assert_eq!(
        common::run_ignore_missing_linters_config(&["python-clean.py", "yaml-clean.yaml"]),
        ""
    );
}

#[test]
fn github_workflow_format() {
    assert_eq!(
        common::run_github_workflow(&["python-clean.py", "yaml-clean.yaml"]),
        "::warning file=python-clean.py::[flake8] linter not found\n\
         ::warning file=python-clean.py::[mypy] linter not found\n\
         ::warning file=python-clean.py::[py_compile] linter not found\n\
         ::warning file=python-clean.py::[pylint] linter not found\n\
         ::warning file=python-clean.py::[pyright] linter not found\n\
         ::warning file=python-clean.py::[ruff] linter not found\n\
         ::warning file=yaml-clean.yaml::[yamllint] linter not found\n\
         Error: lint findings were emitted\n"
    );
}
