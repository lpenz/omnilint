// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of Bazel files, backed by
//! buildifier.
//!
//! Requires `buildifier` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run_clean(&["bazel-clean.bzl"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["bazel-dirty.bzl"]),
        "bazel-dirty.bzl:1: [buildifier] module-docstring: The file has no module docstring.\n\
         bazel-dirty.bzl:1: [buildifier] unused-variable: Variable \"ctx\" is unused. Please remove it.\n"
    );
}
