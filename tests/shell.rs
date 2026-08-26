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
    assert_eq!(
        common::run_with_config_real_path(
            &["shell-clean.sh"],
            "[linters.zsh]\nmode = \"disabled\"\n",
            0,
        ),
        ""
    );
}

#[test]
fn dirty() {
    assert_eq!(
        common::run_with_config_real_path(
            &["shell-dirty.sh"],
            "[linters.zsh]\nmode = \"disabled\"\n",
            1,
        ),
        "Error: lint findings were emitted\n\
         shell-dirty.sh:3: [shellcheck] unused_var appears unused. Verify use (or export if used externally).\n\
         shell-dirty.sh:4: [shellcheck] Double quote to prevent globbing and word splitting.\n\
         shell-dirty.sh:5: [shellcheck] Double quote to prevent globbing and word splitting.\n\
         shell-dirty.sh:5: [shellcheck] missing_var is referenced but not assigned.\n"
    );
}
