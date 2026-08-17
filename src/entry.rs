// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::Result;
use color_eyre::eyre::OptionExt;
use std::fmt;
use std::num::NonZero;
use std::path::{Path, PathBuf};

use crate::cli::OutputFormat;

/// The `Entry` type captures an issue discovered by a lint tool.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Entry {
    filename: PathBuf,
    msg: String,
    linter: String,
    line: Option<NonZero<u32>>,
    col: Option<NonZero<u32>>,
}

impl Entry {
    pub fn new(filename: &Path, linter: &str, msg: &str) -> Result<Entry> {
        Ok(Entry {
            filename: filename.to_path_buf(),
            msg: msg.to_string(),
            linter: linter.to_string(),
            line: None,
            col: None,
        })
    }

    pub fn new_line(filename: &Path, linter: &str, msg: &str, line: u32) -> Result<Entry> {
        Ok(Entry {
            filename: filename.to_path_buf(),
            msg: msg.to_string(),
            linter: linter.to_string(),
            line: Some(NonZero::new(line).ok_or_eyre("line can't be zero")?),
            col: None,
        })
    }

    pub fn new_line_col(
        filename: &Path,
        linter: &str,
        msg: &str,
        line: u32,
        col: u32,
    ) -> Result<Entry> {
        Ok(Entry {
            filename: filename.to_path_buf(),
            msg: msg.to_string(),
            linter: linter.to_string(),
            line: Some(NonZero::new(line).ok_or_eyre("line can't be zero")?),
            col: Some(NonZero::new(col).ok_or_eyre("col can't be zero")?),
        })
    }

    /// Formats the entry according to the given output format.
    pub fn format_output(&self, format: OutputFormat) -> String {
        match format {
            OutputFormat::Default => self.to_string(),
            OutputFormat::GithubWorkflow => {
                let filename = self.filename.display();
                let mut params = format!("file={filename}");
                if let Some(line) = self.line {
                    params.push_str(&format!(",line={line}"));
                }
                if let Some(col) = self.col {
                    params.push_str(&format!(",col={col}"));
                }
                format!("::warning {params}::[{}] {}", self.linter, self.msg)
            }
        }
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:", self.filename.display())?;
        if let Some(line) = self.line {
            write!(f, "{}:", line)?;
        }
        write!(f, " [{}] {}", self.linter, self.msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use color_eyre::Result;

    #[test]
    fn new_basic() -> Result<()> {
        let e = Entry::new(Path::new("foo.rs"), "test", "warning")?;
        assert_eq!(e.to_string(), "foo.rs: [test] warning");
        assert_eq!(
            e.format_output(OutputFormat::GithubWorkflow),
            "::warning file=foo.rs::[test] warning"
        );
        Ok(())
    }

    #[test]
    fn new_line_basic() -> Result<()> {
        let e = Entry::new_line(Path::new("foo.rs"), "test", "error", 10)?;
        assert_eq!(e.to_string(), "foo.rs:10: [test] error");
        assert_eq!(
            e.format_output(OutputFormat::GithubWorkflow),
            "::warning file=foo.rs,line=10::[test] error"
        );
        Ok(())
    }

    #[test]
    fn new_line_zero_line_fails() {
        assert!(Entry::new_line(Path::new("foo.rs"), "test", "error", 0).is_err());
    }

    #[test]
    fn new_line_col_basic() -> Result<()> {
        let e = Entry::new_line_col(Path::new("foo.rs"), "test", "error", 10, 5)?;
        assert_eq!(e.to_string(), "foo.rs:10: [test] error");
        assert_eq!(
            e.format_output(OutputFormat::GithubWorkflow),
            "::warning file=foo.rs,line=10,col=5::[test] error"
        );
        Ok(())
    }

    #[test]
    fn new_line_col_zero_line_fails() {
        assert!(Entry::new_line_col(Path::new("foo.rs"), "test", "error", 0, 5).is_err());
    }

    #[test]
    fn new_line_col_zero_col_fails() {
        assert!(Entry::new_line_col(Path::new("foo.rs"), "test", "error", 10, 0).is_err());
    }
}
