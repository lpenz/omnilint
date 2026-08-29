// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the `inventory` command.

mod common;

#[test]
fn inventory_lists_linters() {
    let output = common::run_inventory();
    assert!(output.contains("actionlint"));
    assert!(output.contains("shellcheck"));
    assert!(output.contains("yamllint"));
}

#[test]
fn inventory_disabled_linters() {
    let output = common::run_inventory_with_config("[linters.flake8]\nmode = \"disabled\"\n");
    assert!(output.contains("flake8"));
    assert!(output.contains("disabled"));
}

#[test]
fn inventory_exit_code() {
    // inventory always exits 0
    common::run_inventory();
}

#[test]
fn inventory_required_not_found_exits_error() {
    let output = common::run_inventory_with_config_and_empty_path(
        "[linters.shellcheck]\nmode = \"required\"\n",
        1,
    );
    assert!(output.contains("required linter 'shellcheck' not found"));
    assert!(output.contains("shellcheck"));
    assert!(output.contains("yamllint"));
}

#[test]
fn inventory_required_default_mode_not_found_exits_error() {
    let output = common::run_inventory_with_config_and_empty_path(
        "[global]\ndefault_linter_mode = \"required\"\n",
        1,
    );
    assert!(output.contains("required linter 'shellcheck' not found"));
    assert!(output.contains("shellcheck"));
    assert!(output.contains("yamllint"));
}

#[test]
fn inventory_wanted_not_found_still_exits_zero() {
    // default mode is wanted, so a missing linter is not an error
    common::run_inventory_with_config_and_empty_path("", 0);
}
