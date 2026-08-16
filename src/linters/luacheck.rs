// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [luacheck](https://luacheck.readthedocs.io/) Lua linter wrapper.
//!
//! luacheck checks Lua source files for style errors, unused variables,
//! and other issues. It is run once per file with `--formatter plain` to
//! produce machine-readable output that is parsed into [`Entry`] values.
//!
//! ## Output format
//!
//! Each line emitted by luacheck on stdout has the form:
//!
//! ```text
//! <filename>:<line>:<col>: <message>
//! ```
//!
//! For example:
//!
//! ```text
//! foo.lua:1:7: unused variable 'unused'
//! ```

use crate::entry::Entry;
use crate::linters::{Linter, Linters, parse_line_standard};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct LuaLuacheck {
    filename: PathBuf,
    inner: Linter,
}

impl LuaLuacheck {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let mut cmd = Command::new("luacheck");
        cmd.arg("--formatter");
        cmd.arg("plain");
        cmd.arg(filename);
        let inner = linters.spawn("luacheck", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        parse_line_standard(filename, "luacheck", line)
    }
}

impl Stream for LuaLuacheck {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "luacheck",
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
        let entry = LuaLuacheck::parse_line(
            Path::new("test.lua"),
            "test.lua:1:7: unused variable 'unused'",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.lua:1: [luacheck] unused variable 'unused'"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(LuaLuacheck::parse_line(Path::new("test.lua"), "").is_none());
    }

    #[test]
    fn parse_line_too_few_parts() {
        assert!(LuaLuacheck::parse_line(Path::new("test.lua"), "no colons here").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(LuaLuacheck::parse_line(Path::new("test.lua"), "test.lua:x:y: msg").is_none());
    }
}
