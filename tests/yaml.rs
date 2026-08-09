// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of YAML files, backed by yamllint.
//!
//! Requires `yamllint` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run(&["yaml-clean.yaml"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["yaml-dirty.yaml"]),
        "yaml-dirty.yaml:1: [yamllint] missing document start \"---\"\n\
         yaml-dirty.yaml:1: [yamllint] trailing spaces\n\
         yaml-dirty.yaml:2: [yamllint] duplication of key \"foo\" in mapping\n"
    );
}
