// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of Swift files, backed by swiftlint.
//!
//! Requires `swiftlint` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run_clean(&["swift-clean.swift"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["swift-dirty.swift"]),
        "Error: lint findings were emitted\n\
         swift-dirty.swift:3: [swiftlint] Identifier Name Violation: Variable name 'x' should be between 3 and 40 characters long (identifier_name)\n"
    );
}
