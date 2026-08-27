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
pub fn fixtures_dir() -> PathBuf {
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

/// Runs `omnilint files --default-linter-mode optional` on the given files
/// with a `PATH` that contains no linter tools.
///
/// Asserts that omnilint exits with status 0, since the missing linters are
/// optional.
pub fn run_ignore_missing_linters(files: &[&str]) -> String {
    run_command(
        "files",
        files,
        &[("PATH", "/nonexistent")],
        0,
        &["--default-linter-mode", "optional"],
    )
}

/// Runs `omnilint files` on the given files with a `PATH` that contains no
/// linter tools and a config that sets `default_linter_mode = "optional"`.
///
/// Asserts that omnilint exits with status 0, since the missing linters are
/// optional via config.
pub fn run_ignore_missing_linters_config(files: &[&str]) -> String {
    run_with_config(files, "[global]\ndefault_linter_mode = \"optional\"\n", 0)
}

/// Runs `omnilint files` with `--format github-workflow` on the given files
/// with a `PATH` that contains no linter tools.
///
/// Asserts that omnilint exits with status 1, since missing linters count as
/// issues.
pub fn run_github_workflow(files: &[&str]) -> String {
    run_command(
        "files",
        files,
        &[("PATH", "/nonexistent")],
        1,
        &["--format", "github-workflow"],
    )
}

/// Runs `omnilint files` on the given files from a temporary directory that
/// contains an `omnilint.toml` with the given contents and a `PATH` that has
/// no linter tools.
///
/// Asserts that omnilint exits with the given status code.
pub fn run_with_config(files: &[&str], config_contents: &str, code: i32) -> String {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("omnilint.toml"), config_contents).unwrap();
    for file in files {
        std::fs::copy(fixtures_dir().join(file), tmp.path().join(file)).unwrap();
    }
    let mut cmd = Command::cargo_bin("omnilint").unwrap();
    let mut command = cmd
        .current_dir(tmp.path())
        .env_remove("OMNILINT_CONFIG")
        .arg("files");
    for file in files {
        command = command.arg(file);
    }
    command = command.env("PATH", "/nonexistent");
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

/// Runs `omnilint files` on the given files with `OMNILINT_CONFIG` set to
/// the given path and a `PATH` that has no linter tools.
///
/// Asserts that omnilint exits with the given status code.
pub fn run_with_config_env(files: &[&str], config_path: &str, code: i32) -> String {
    let mut cmd = Command::cargo_bin("omnilint").unwrap();
    let mut command = cmd
        .current_dir(fixtures_dir())
        .env("OMNILINT_CONFIG", config_path)
        .arg("files");
    for file in files {
        command = command.arg(file);
    }
    command = command.env("PATH", "/nonexistent");
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

/// Runs `omnilint files --config <path>` on the given files with a `PATH`
/// that has no linter tools.
///
/// Asserts that omnilint exits with the given status code.
pub fn run_with_config_flag(files: &[&str], config_path: &str, code: i32) -> String {
    let mut cmd = Command::cargo_bin("omnilint").unwrap();
    let mut command = cmd
        .current_dir(fixtures_dir())
        .env_remove("OMNILINT_CONFIG")
        .arg("files")
        .arg("--config")
        .arg(config_path);
    for file in files {
        command = command.arg(file);
    }
    command = command.env("PATH", "/nonexistent");
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
        .env_remove("OMNILINT_CONFIG")
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

/// Runs `omnilint inventory` and returns its stderr.
///
/// Asserts that omnilint exits with status 0.
pub fn run_inventory() -> String {
    let mut cmd = Command::cargo_bin("omnilint").unwrap();
    let command = cmd
        .current_dir(fixtures_dir())
        .env_remove("OMNILINT_CONFIG")
        .arg("inventory");
    let output = command.assert().code(0).stdout("");
    String::from_utf8_lossy(&output.get_output().stderr).to_string()
}

/// Runs `omnilint inventory` from a temporary directory with the given config
/// and returns its stderr.
///
/// Asserts that omnilint exits with status 0.
pub fn run_inventory_with_config(config_contents: &str) -> String {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("omnilint.toml"), config_contents).unwrap();
    let mut cmd = Command::cargo_bin("omnilint").unwrap();
    let command = cmd
        .current_dir(tmp.path())
        .env_remove("OMNILINT_CONFIG")
        .arg("inventory");
    let output = command.assert().code(0).stdout("");
    String::from_utf8_lossy(&output.get_output().stderr).to_string()
}
