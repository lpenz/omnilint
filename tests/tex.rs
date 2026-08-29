// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of TeX files, backed by chktex.
//!
//! Requires `chktex` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run_clean(&["tex-clean.tex"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["tex-dirty.tex"]),
        "Error: lint findings were emitted\n\
         tex-dirty.tex:3: [chktex] Use ' to end quotation, not `.\n"
    );
}
