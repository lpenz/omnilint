// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of JSON files by the built-in
//! `json-parse` linter, which parses the file contents in-process using the
//! [`serde_json`](https://crates.io/crates/serde_json) crate, together with
//! the external `jq` linter.
//!
//! Requires `jq` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run_clean(&["json-clean.json"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["json-dirty.json"]),
        "Error: lint findings were emitted\n\
         json-dirty.json:1: [jq] Unmatched '}'\n\
         json-dirty.json:1: [json-parse] expected value at line 1 column 15\n"
    );
}
