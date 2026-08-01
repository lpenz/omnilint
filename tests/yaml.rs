// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of YAML files, backed by yamllint.
//!
//! Requires `yamllint` to be available on the `PATH`.

mod common;

#[test]
fn clean() {
    assert_eq!(common::run(&["yaml-clean.yaml"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["yaml-dirty.yaml"]),
        "yaml-dirty.yaml:1:1: missing document start \"---\"\n\
         yaml-dirty.yaml:1:9: trailing spaces\n\
         yaml-dirty.yaml:2:1: duplication of key \"foo\" in mapping\n"
    );
}
