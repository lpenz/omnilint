// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of shell scripts, backed by
//! shellcheck.
//!
//! Requires `shellcheck` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run(&["shell-clean.sh"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["shell-dirty.sh"]),
        "shell-dirty.sh:3: [shellcheck] unused_var appears unused. Verify use (or export if used externally).\n\
         shell-dirty.sh:4: [shellcheck] Double quote to prevent globbing and word splitting.\n\
         shell-dirty.sh:5: [shellcheck] Double quote to prevent globbing and word splitting.\n\
         shell-dirty.sh:5: [shellcheck] missing_var is referenced but not assigned.\n"
    );
}
