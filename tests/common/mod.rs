// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Shared helpers for the integration tests.

// Each integration test compiles this module into its own binary and only
// uses a subset of the helpers, so the others would be reported as dead code.
#![allow(dead_code)]

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
    run_command("files", files, &[])
}

/// Runs `omnilint repository` in the fixtures directory (a git repository)
/// and returns its stderr, with the output lines sorted so that the result
/// is deterministic even when linters run in parallel.
pub fn run_repository() -> String {
    run_command("repository", &[], &[])
}

/// Runs `omnilint files` on the given files with a `PATH` that contains no
/// linter tools, so that every linter reports it was not found.
pub fn run_without_linters(files: &[&str]) -> String {
    run_command("files", files, &[("PATH", "/nonexistent")])
}

fn run_command(subcommand: &str, files: &[&str], envs: &[(&str, &str)]) -> String {
    let mut cmd = Command::cargo_bin("omnilint").unwrap();
    let mut command = cmd.current_dir(fixtures_dir()).arg(subcommand);
    for file in files {
        command = command.arg(file);
    }
    for (key, value) in envs {
        command = command.env(key, value);
    }
    let output = command.assert().success().stdout("");
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
