// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of systemd unit files, backed by
//! `systemd-analyze verify`.
//!
//! Requires `systemd-analyze` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run_clean(&["systemd-clean.service"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["systemd-dirty.service"]),
        "Error: lint findings were emitted\n\
         systemd-dirty.service:5: [systemd-analyze] Unknown key 'Foo' in section [Service], ignoring.\n"
    );
}
