// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of JavaScript and TypeScript files,
//! backed by oxlint.
//!
//! Requires `oxlint` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean_js() {
    assert_eq!(common::run_clean(&["js-clean.js"]), "");
}

#[test]
fn clean_ts() {
    assert_eq!(common::run_clean(&["ts-clean.ts"]), "");
}

#[test]
fn dirty_js() {
    assert_eq!(
        common::run(&["js-dirty.js"]),
        "Error: lint findings were emitted\n\
         js-dirty.js:1: [oxlint] Identifier expected. 'debugger' is a reserved word that cannot be used here. [Error]\n"
    );
}

#[test]
fn dirty_ts() {
    assert_eq!(
        common::run(&["ts-dirty.ts"]),
        "Error: lint findings were emitted\n\
         ts-dirty.ts:1: [oxlint] Identifier expected. 'debugger' is a reserved word that cannot be used here. [Error]\n"
    );
}
