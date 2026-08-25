// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of Lua files, backed by luacheck
//! and luau-analyze.
//!
//! Requires `luacheck` and/or `luau-analyze` to be available on the
//! `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean_luacheck() {
    assert_eq!(
        common::run_with_config_real_path(
            &["lua-clean.lua"],
            "[linters.luau-analyze]\nmode = \"disabled\"\n",
            0,
        ),
        ""
    );
}

#[test]
fn dirty_luacheck() {
    assert_eq!(
        common::run_with_config_real_path(
            &["lua-dirty.lua"],
            "[linters.luau-analyze]\nmode = \"disabled\"\n",
            1,
        ),
        "Error: lint findings were emitted\n\
         lua-dirty.lua:1: [luacheck] unused variable 'unused'\n"
    );
}

#[test]
fn clean_luau() {
    assert_eq!(
        common::run_with_config_real_path(&["luau-clean.luau"], "", 0),
        ""
    );
}

#[test]
fn dirty_luau() {
    let output = common::run_with_config_real_path(&["luau-dirty.luau"], "", 1);
    assert!(output.contains("[luau-analyze]"));
}
