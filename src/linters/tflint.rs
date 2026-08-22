// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [tflint](https://github.com/terraform-linters/tflint) Terraform linter
//! wrapper.
//!
//! tflint checks Terraform source files for errors and best practice
//! violations. It is run with `--format=compact` and `--filter=<file>` so
//! that only the given file is analysed, and its output is parsed into
//! [`Entry`] values.//!
//! ## Output format
//!
//! Each finding is a single line, preceded by a header line that is
//! skipped:
//!
//! ```text
//! 1 issue(s) found:
//!
//! foo.tf:1:1: Warning - Missing version constraint for provider "local" in `required_providers` (terraform_required_providers)
//! ```
//!
//! ## Isolated execution directory
//!
//! tflint analyses the whole module in the working directory and attributes
//! the findings of some rules (such as `terraform_required_providers`) to
//! an arbitrary file of the module - which file it is varies between runs,
//! as tflint is a Go program and Go randomizes map iteration order. To make
//! the analysis of a given file deterministic, each file is copied to its
//! own temporary directory, where it is analysed as a single-file module.

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct TfTflint {
    filename: PathBuf,
    inner: Linter,
    #[allow(dead_code)]
    tempdir: tempfile::TempDir,
}

impl TfTflint {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("tflint");
        let tempdir = tempfile::tempdir()?;
        let file_name = filename
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| filename.to_string_lossy().to_string());
        std::fs::copy(filename, tempdir.path().join(&file_name))?;
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("--format=compact")
            .current_dir(tempdir.path())
            .arg(format!("--filter={file_name}"));
        let inner = linters.spawn("tflint", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
            tempdir,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        let line = line.trim();
        let parts: Vec<&str> = line.splitn(4, ':').collect();
        if parts.len() < 4 {
            return None;
        }
        let line_num: u32 = parts[1].trim().parse().ok()?;
        let col_num: u32 = parts[2].trim().parse().ok()?;
        let rest = parts[3].trim();
        for severity in ["Error - ", "Warning - ", "Notice - "] {
            if let Some(msg) = rest.strip_prefix(severity) {
                return Some(
                    Entry::new_line_col(filename, "tflint", msg, line_num, col_num).unwrap(),
                );
            }
        }
        None
    }
}

impl Stream for TfTflint {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "tflint",
            &this.filename,
            &mut this.inner,
            Self::parse_line,
            false,
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_standard() {
        let entry = TfTflint::parse_line(
            Path::new("test.tf"),
            r#"test.tf:1:1: Warning - Missing version constraint for provider "local" (terraform_required_providers)"#,
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.tf:1: [tflint] Missing version constraint for provider \"local\" (terraform_required_providers)"
        );
    }

    #[test]
    fn parse_line_error() {
        let entry = TfTflint::parse_line(
            Path::new("test.tf"),
            "test.tf:3:5: Error - something failed",
        )
        .unwrap();
        assert_eq!(entry.to_string(), "test.tf:3: [tflint] something failed");
    }

    #[test]
    fn parse_line_skips_header() {
        assert!(TfTflint::parse_line(Path::new("test.tf"), "1 issue(s) found:").is_none());
    }

    #[test]
    fn parse_line_empty() {
        assert!(TfTflint::parse_line(Path::new("test.tf"), "").is_none());
    }

    #[test]
    fn parse_line_too_few_parts() {
        assert!(TfTflint::parse_line(Path::new("test.tf"), "no colons here").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(TfTflint::parse_line(Path::new("test.tf"), "test.tf:x:y: Error - msg").is_none());
    }
}
