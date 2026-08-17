// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the TOML config file support.

mod common;

#[test]
fn disabled_linter_skips_output() {
    assert_eq!(
        common::run_with_config(
            &["python-clean.py"],
            "[linters.flake8]\ndisabled = true\n",
            1,
        ),
        "python-clean.py: [ruff] linter not found\n"
    );
}

#[test]
fn disabled_linter_does_not_affect_exit_status_when_ignored() {
    assert_eq!(
        common::run_with_config(
            &["python-clean.py"],
            "[global]\nignore_missing_linters = true\n",
            0,
        ),
        ""
    );
}

#[test]
fn disabled_linter_combined_with_ignore_missing() {
    assert_eq!(
        common::run_with_config(
            &["python-clean.py"],
            "[global]\nignore_missing_linters = true\n\n[linters.flake8]\ndisabled = true\n",
            0,
        ),
        ""
    );
}
