// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of TOML files by the built-in
//! `toml-parse` linter, which parses the file contents in-process using the
//! [`toml`](https://crates.io/crates/toml) crate. Unlike the external linters,
//! it requires no tool on the `PATH`, so the tests also run without the
//! `test-linter-tools` feature.

mod common;

#[test]
fn clean() {
    assert_eq!(common::run_clean(&["toml-clean.toml"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["toml-dirty.toml"]),
        "Error: lint findings were emitted\n\
         toml-dirty.toml:6: [toml-parse] duplicate key `server` in document root\n"
    );
}
