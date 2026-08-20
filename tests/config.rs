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
            "[linters.flake8]\nmode = \"disabled\"\n",
            1,
        ),
        "python-clean.py: [pylint] linter not found\n\
         python-clean.py: [ruff] linter not found\n"
    );
}

#[test]
fn optional_linter_skips_when_not_found() {
    assert_eq!(
        common::run_with_config(
            &["python-clean.py"],
            "[global]\ndefault_linter_mode = \"optional\"\n",
            0,
        ),
        ""
    );
}

#[test]
fn disabled_linter_combined_with_optional() {
    assert_eq!(
        common::run_with_config(
            &["python-clean.py"],
            "[global]\ndefault_linter_mode = \"optional\"\n\n[linters.flake8]\nmode = \"disabled\"\n",
            0,
        ),
        ""
    );
}

#[test]
fn linter_custom_path() {
    assert_eq!(
        common::run_with_config(
            &["python-clean.py"],
            "[linters.flake8]\npath = \"/nonexistent/flake8\"\n",
            1,
        ),
        "python-clean.py: [flake8] linter not found\n\
         python-clean.py: [pylint] linter not found\n\
         python-clean.py: [ruff] linter not found\n"
    );
}

#[test]
fn omnilint_config_env_var() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join("disabled.toml"),
        "[linters.flake8]\nmode = \"disabled\"\n",
    )
    .unwrap();
    assert_eq!(
        common::run_with_config_env(
            &["python-clean.py"],
            tmp.path().join("disabled.toml").to_str().unwrap(),
            1,
        ),
        "python-clean.py: [pylint] linter not found\n\
         python-clean.py: [ruff] linter not found\n"
    );
}
