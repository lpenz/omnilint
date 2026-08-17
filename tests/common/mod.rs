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
///
/// Asserts that omnilint exits with status 1, i.e. that at least one issue
/// was found.
pub fn run(files: &[&str]) -> String {
    run_command("files", files, &[], 1, &[])
}

/// Runs `omnilint files` on the given files, asserting that omnilint exits
/// with status 0, i.e. that no issues were found.
pub fn run_clean(files: &[&str]) -> String {
    run_command("files", files, &[], 0, &[])
}

/// Runs `omnilint repository` in the fixtures directory (a git repository)
/// and returns its stderr, with the output lines sorted so that the result
/// is deterministic even when linters run in parallel.
///
/// Asserts that omnilint exits with status 1, i.e. that at least one issue
/// was found.
pub fn run_repository() -> String {
    run_command("repository", &[], &[], 1, &[])
}

/// Runs `omnilint files` on the given files with a `PATH` that contains no
/// linter tools, so that every linter reports it was not found.
///
/// Asserts that omnilint exits with status 1, since a missing linter counts
/// as an issue.
pub fn run_without_linters(files: &[&str]) -> String {
    run_command("files", files, &[("PATH", "/nonexistent")], 1, &[])
}

/// Runs `omnilint files --ignore-missing-linters` on the given files with a
/// `PATH` that contains no linter tools.
///
/// Asserts that omnilint exits with status 0, since the missing linters are
/// ignored.
pub fn run_ignore_missing_linters(files: &[&str]) -> String {
    run_command(
        "files",
        files,
        &[("PATH", "/nonexistent")],
        0,
        &["--ignore-missing-linters"],
    )
}

/// Runs `omnilint files` on the given files with a `PATH` that contains no
/// linter tools and with `OMNILINT_IGNORE_MISSING_LINTERS=1` set in the
/// environment.
///
/// Asserts that omnilint exits with status 0, since the missing linters are
/// ignored via the environment variable.
pub fn run_ignore_missing_linters_env(files: &[&str]) -> String {
    run_command(
        "files",
        files,
        &[
            ("PATH", "/nonexistent"),
            ("OMNILINT_IGNORE_MISSING_LINTERS", "1"),
        ],
        0,
        &[],
    )
}

fn run_command(
    subcommand: &str,
    files: &[&str],
    envs: &[(&str, &str)],
    code: i32,
    args: &[&str],
) -> String {
    let mut cmd = Command::cargo_bin("omnilint").unwrap();
    let mut command = cmd
        .current_dir(fixtures_dir())
        .env_remove("OMNILINT_IGNORE_MISSING_LINTERS")
        .arg(subcommand);
    for file in files {
        command = command.arg(file);
    }
    for arg in args {
        command = command.arg(arg);
    }
    for (key, value) in envs {
        command = command.env(key, value);
    }
    let output = command.assert().code(code).stdout("");
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
