// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of Lua files, backed by luacheck.
//!
//! Requires `luacheck` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run(&["lua-clean.lua"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["lua-dirty.lua"]),
        "lua-dirty.lua:1: [luacheck] unused variable 'unused'\n"
    );
}
