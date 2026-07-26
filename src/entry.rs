// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::Result;
use color_eyre::eyre::OptionExt;
use std::fmt;
use std::num::NonZero;
use std::path::{Path, PathBuf};

/// The `Entry` type captures an issue discovered by a lint tool.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Entry {
    filename: PathBuf,
    msg: String,
    line: Option<NonZero<u32>>,
    col: Option<NonZero<u32>>,
}

impl Entry {
    pub fn new(filename: &Path, msg: &str) -> Result<Entry> {
        Ok(Entry {
            filename: filename.to_path_buf(),
            msg: msg.to_string(),
            line: None,
            col: None,
        })
    }

    pub fn new_line_col(filename: &Path, msg: &str, line: u32, col: u32) -> Result<Entry> {
        Ok(Entry {
            filename: filename.to_path_buf(),
            msg: msg.to_string(),
            line: Some(NonZero::new(line).ok_or_eyre("line can't be zero")?),
            col: Some(NonZero::new(col).ok_or_eyre("col can't be zero")?),
        })
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:", self.filename.display())?;
        if let Some(line) = self.line {
            write!(f, "{}:", line)?;
            if let Some(col) = self.col {
                write!(f, "{}:", col)?;
            }
        }
        write!(f, " {}", self.msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use color_eyre::Result;

    #[test]
    fn new_basic() -> Result<()> {
        let e = Entry::new(Path::new("foo.rs"), "warning")?;
        assert_eq!(e.to_string(), "foo.rs: warning");
        Ok(())
    }

    #[test]
    fn new_line_col_basic() -> Result<()> {
        let e = Entry::new_line_col(Path::new("foo.rs"), "error", 10, 5)?;
        assert_eq!(e.to_string(), "foo.rs:10:5: error");
        Ok(())
    }

    #[test]
    fn new_line_col_zero_line_fails() {
        assert!(Entry::new_line_col(Path::new("foo.rs"), "error", 0, 5).is_err());
    }

    #[test]
    fn new_line_col_zero_col_fails() {
        assert!(Entry::new_line_col(Path::new("foo.rs"), "error", 10, 0).is_err());
    }
}
