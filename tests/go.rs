// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of Go files, backed by staticcheck
//! and go vet.
//!
//! Requires `staticcheck` and `go` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run_clean(&["go-clean.go"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["go-dirty.go"]),
        "Error: lint findings were emitted\n\
         go-dirty.go:5: [staticcheck] func unused is unused (U1000)\n\
         go-dirty.go:6: [go-vet] fmt.Printf format %s reads arg #1, but call has 0 args\n\
         go-dirty.go:6: [staticcheck] Printf format %s reads arg #1, but call has only 0 args (SA5009)\n"
    );
}
