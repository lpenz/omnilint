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
        "lua-dirty.lua:1: [luacheck] unused variable 'unused'\n"
    );
}

#[test]
fn clean_luau() {
    assert_eq!(
        common::run_with_config_real_path(
            &["luau-clean.luau"],
            "[linters.luacheck]\nmode = \"disabled\"\n",
            0,
        ),
        ""
    );
}

#[test]
fn dirty_luau() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join("omnilint.toml"),
        "[linters.luacheck]\nmode = \"disabled\"\n",
    )
    .unwrap();
    std::fs::copy(
        common::fixtures_dir().join("luau-dirty.luau"),
        tmp.path().join("luau-dirty.luau"),
    )
    .unwrap();
    let output = assert_cmd::Command::cargo_bin("omnilint")
        .unwrap()
        .current_dir(tmp.path())
        .env_remove("OMNILINT_CONFIG")
        .args(["files", "luau-dirty.luau"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let code = output.status.code().unwrap_or(-1);
    assert!(code == 0 || code == 1, "unexpected exit code: {code}");
    assert!(
        stderr.contains("[luau-analyze]") || stderr.is_empty(),
        "unexpected stderr: {stderr}"
    );
}
