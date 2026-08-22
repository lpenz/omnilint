// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of Terraform files, backed by tflint.
//!
//! Requires `tflint` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run_clean(&["terraform-clean.tf"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["terraform-dirty.tf"]),
        "terraform-dirty.tf:1: [tflint] Missing version constraint for provider \"local\" in `required_providers` (terraform_required_providers)\n\
         terraform-dirty.tf:1: [tflint] terraform \"required_version\" attribute is required (terraform_required_version)\n"
    );
}
