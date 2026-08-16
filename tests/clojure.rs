// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of Clojure files, backed by clj-kondo.
//!
//! Requires `clj-kondo` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run_clean(&["clj-clean.clj"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["clj-dirty.clj"]),
        "clj-dirty.clj:1: [clj-kondo] unused binding x\n\
         clj-dirty.clj:2: [clj-kondo] Unresolved symbol: y\n\
         clj-dirty.clj:2: [clj-kondo] unused binding unused\n"
    );
}
