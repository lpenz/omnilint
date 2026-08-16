// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of Perl files, backed by perlcritic.
//!
//! Requires `perlcritic` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run_clean(&["perl-clean.pl"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["perl-dirty.pl"]),
        "perl-dirty.pl:1: [perlcritic] Code before strictures are enabled\n"
    );
}
