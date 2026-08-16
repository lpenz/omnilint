// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of SQL files, backed by sqlfluff.
//!
//! Requires `sqlfluff` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run_clean(&["sql-clean.sql"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["sql-dirty.sql"]),
        "sql-dirty.sql:1: [sqlfluff] AM04: Query produces an unknown number of result columns. [ambiguous.column_count]\n"
    );
}
