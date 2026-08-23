// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of R files, backed by lintr.
//!
//! Requires `Rscript` with the `lintr` package to be available on the
//! `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run_clean(&["r-clean.R"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["r-dirty.R"]),
        "r-dirty.R:1: [lintr] style: [assignment_linter] Use one of <-, <<- for assignment, not =.\n\
         r-dirty.R:2: [lintr] style: [return_linter] Use implicit return behavior; explicit return() is not needed.\n"
    );
}
