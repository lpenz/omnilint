// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of Haskell files, backed by hlint.
//!
//! Requires `hlint` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run_clean(&["haskell-clean.hs"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["haskell-dirty.hs"]),
        "haskell-dirty.hs:12: [hlint] Warning: Eta reduce\n\
         haskell-dirty.hs:12: [hlint] Warning: Use concat\n\
         haskell-dirty.hs:8: [hlint] Suggestion: Replace case with fromMaybe\n"
    );
}
