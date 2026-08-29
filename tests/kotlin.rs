// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of Kotlin files, backed by ktlint.
//!
//! Requires `ktlint` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run_clean(&["Clean.kt"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["Dirty.kt"]),
        "Dirty.kt:2: [ktlint] Unnecessary semicolon (standard:no-semi)\n\
         Error: lint findings were emitted\n"
    );
}
