// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of VHDL files, backed by `ghdl -a`.
//!
//! Requires `ghdl` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run_clean(&["vhdl-clean.vhd"]), "");
}

#[test]
fn dirty() {
    let output = common::run(&["vhdl-dirty.vhd"]);
    let lines: Vec<&str> = output.lines().collect();
    // GHDL reports two errors for this file: port "a" can't be assigned and
    // port "b" cannot be read.  The exact order depends on the GHDL version,
    // so just verify both are present.
    assert_eq!(lines.len(), 2);
    assert!(lines.iter().all(|l| l.contains("[ghdl]")));
    assert!(
        lines
            .iter()
            .any(|l| l.contains("port \"a\" can't be assigned"))
    );
    assert!(
        lines
            .iter()
            .any(|l| l.contains("port \"b\" cannot be read"))
    );
}
