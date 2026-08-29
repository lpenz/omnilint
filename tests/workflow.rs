// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of GitHub Actions workflow files,
//! backed by yamllint and actionlint.
//!
//! Requires `yamllint` and `actionlint` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(
        common::run(&[".github/workflows/clean.yml"]),
        ".github/workflows/clean.yml:3: [yamllint] truthy value should be one of [false, true]\n\
         Error: lint findings were emitted\n"
    );
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&[".github/workflows/dirty.yml"]),
        ".github/workflows/dirty.yml:3: [yamllint] truthy value should be one of [false, true]\n\
         .github/workflows/dirty.yml:8: [actionlint] element of \"steps\" section is scalar node but mapping node is expected\n\
         .github/workflows/dirty.yml:8: [actionlint] step must run script with \"run\" section or run action with \"uses\" section\n\
         Error: lint findings were emitted\n"
    );
}
