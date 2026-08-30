// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of Nix files, backed by statix.
//!
//! Requires `statix` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run_clean(&["nix-clean.nix"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["nix-dirty.nix"]),
        "Error: lint findings were emitted\n\
         nix-dirty.nix:2: [statix] Consider quoting this URI expression\n\
         nix-dirty.nix:3: [statix] Useless parentheses around primitive expression\n"
    );
}
