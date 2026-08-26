// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [luac](https://www.lua.org/) Lua syntax checker.
//!
//! `luac -p` compiles a Lua source file to bytecode without running it,
//! checking for syntax errors. It serves as a basic fallback alongside
//! luacheck and luau-analyze.
//!
//! ## Output format
//!
//! Syntax errors are printed to stderr:
//!
//! ```text
//! luac: <filename>:<line>: <message>
//! ```

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct LuaLuac {
    filename: PathBuf,
    inner: Linter,
}

impl LuaLuac {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("luac");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("-p");
        cmd.arg(filename);
        let inner = linters.spawn("luac", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        let line = line.trim();
        let fname = filename.to_str()?;
        let marker = format!("luac: {fname}:");
        let rest = line.strip_prefix(&marker)?;
        let (line_str, rest) = rest.split_once(':')?;
        let line_num: u32 = line_str.parse().ok()?;
        let msg = rest.trim();
        Some(Entry::new_line(filename, "luac", msg, line_num).unwrap())
    }
}

impl Stream for LuaLuac {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "luac",
            &this.filename,
            &mut this.inner,
            Self::parse_line,
            true,
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_standard() {
        let entry =
            LuaLuac::parse_line(Path::new("test.lua"), "luac: test.lua:3: syntax error").unwrap();
        assert_eq!(entry.to_string(), "test.lua:3: [luac] syntax error");
    }

    #[test]
    fn parse_line_empty() {
        assert!(LuaLuac::parse_line(Path::new("test.lua"), "").is_none());
    }

    #[test]
    fn parse_line_no_prefix() {
        assert!(LuaLuac::parse_line(Path::new("test.lua"), "test.lua:3: syntax error").is_none());
    }
}
