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
