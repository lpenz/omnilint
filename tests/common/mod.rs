// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Shared helpers for the integration tests.

use assert_cmd::Command;
use std::path::{Path, PathBuf};

/// Returns the fixtures directory.
fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Runs `omnilint files` on the given files from the fixtures directory and
/// returns its stderr, with the output lines sorted so that the result is
/// deterministic even when linters run in parallel.
pub fn run(files: &[&str]) -> String {
    let mut cmd = Command::cargo_bin("omnilint").unwrap();
    let output = cmd
        .current_dir(fixtures_dir())
        .arg("files")
        .args(files)
        .assert()
        .success()
        .stdout("");
    let mut lines: Vec<String> = String::from_utf8_lossy(&output.get_output().stderr)
        .lines()
        .map(Into::into)
        .collect();
    if lines.is_empty() {
        return String::new();
    }
    lines.sort();
    lines.join("\n") + "\n"
}
