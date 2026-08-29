// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of CSS files, backed by stylelint.
//!
//! Requires `stylelint` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run_clean(&["css-clean.css"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["css-dirty.css"]),
        "Error: lint findings were emitted\n\
         css-dirty.css:1: [stylelint] Empty block (block-no-empty) [error]\n"
    );
}
