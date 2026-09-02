// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of XML files by the built-in `xml-parse`
//! linter, which parses the file contents in-process using the
//! [`quick-xml`](https://crates.io/crates/quick-xml) crate, together with the
//! external `xmllint` linter.
//!
//! Requires `xmllint` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run_clean(&["xml-clean.xml"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["xml-dirty.xml"]),
        "Error: lint findings were emitted\n\
         xml-dirty.xml: [xml-parse] ill-formed document: expected `</child>`, but `</b>` was found\n\
         xml-dirty.xml:2: [xmllint] Opening and ending tag mismatch: child line 2 and b\n"
    );
}
