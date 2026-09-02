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
use crate::linters::{CommandLinter, Linters, Spec, parse_line_standard};

use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio_stream::Stream;

pub struct LuaLuacheck(CommandLinter);

impl LuaLuacheck {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        Ok(Self(CommandLinter::new(
            linters,
            Spec {
                name: "luacheck",
                args: &["--formatter", "plain"],
                parse: parse_line,
                ..Default::default()
            },
            filename,
        )?))
    }
}

fn parse_line(filename: &Path, line: &str) -> Vec<Entry> {
    parse_line_standard(filename, "luacheck", line)
}

linter_stream!(LuaLuacheck);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_standard() {
        let entries = parse_line(
            Path::new("test.lua"),
            "test.lua:1:7: unused variable 'unused'",
        );
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].to_string(),
            "test.lua:1: [luacheck] unused variable 'unused'"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(parse_line(Path::new("test.lua"), "").is_empty());
    }

    #[test]
    fn parse_line_too_few_parts() {
        assert!(parse_line(Path::new("test.lua"), "no colons here").is_empty());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(parse_line(Path::new("test.lua"), "test.lua:x:y: msg").is_empty());
    }
}
