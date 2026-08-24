// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of Verilog/SystemVerilog files,
//! backed by `verilator --lint-only`.
//!
//! Requires `verilator` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run_clean(&["verilog-clean.sv"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["verilog-dirty.sv"]),
        "verilog-dirty.sv:5: [verilator] Operator ASSIGNW expects 4 bits on the Assign RHS, but Assign RHS's CONST '8'hff' generates 8 bits.\n"
    );
}
