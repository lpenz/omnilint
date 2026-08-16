// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of Dockerfiles, backed by hadolint.
//!
//! Requires `hadolint` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run_clean(&["docker-clean.dockerfile"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["docker-dirty.dockerfile"]),
        "docker-dirty.dockerfile:1: [hadolint] Using latest is prone to errors if the image will ever update. Pin the version explicitly to a release tag\n\
         docker-dirty.dockerfile:2: [hadolint] Delete the apt lists (/var/lib/apt/lists) after installing something\n\
         docker-dirty.dockerfile:3: [hadolint] Avoid additional packages by specifying `--no-install-recommends`\n\
         docker-dirty.dockerfile:3: [hadolint] Multiple consecutive `RUN` instructions. Consider consolidation.\n\
         docker-dirty.dockerfile:3: [hadolint] Pin versions in apt get install. Instead of `apt-get install <package>` use `apt-get install <package>=<version>`\n"
    );
}
