// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of Ruby files, backed by rubocop and
//! standardrb.
//!
//! Requires `rubocop` and `standardrb` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run_clean(&["ruby-clean.rb"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["ruby-dirty.rb"]),
        "ruby-dirty.rb:1: [rubocop] C: [Correctable] Style/FrozenStringLiteralComment: Missing frozen string literal comment.\n"
    );
}

#[test]
fn standard_dirty() {
    assert_eq!(
        common::run(&["ruby-standard-dirty.rb"]),
        "ruby-standard-dirty.rb:3: [standardrb] C: [Correctable] Style/StringLiterals: Prefer double-quoted strings unless you need single quotes to avoid extra backslashes for escaping.\n"
    );
}
