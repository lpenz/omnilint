// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [luau-analyze](https://github.com/luau-lang/luau) Luau linter wrapper.
//!
//! luau-analyze is the command-line type checker and linter for the
//! [Luau](https://luau.org/) programming language. It checks Luau source
//! files for type errors, syntax errors, and lint warnings. Each file is
//! analysed independently.
//!
//! ## Output format
//!
//! By default, luau-analyze prints findings to stderr in the form:
//!
//! ```text
//! <name>(<line>,<col>): <type>: <message>
//! ```
//!
//! For example:
//!
//! ```text
//! foo.luau(3,7): TypeError: Type 'nil' could be converted into type 'string'
//! ```
//!
//! The `<name>` component is the module name as reported by luau-analyze,
//! which is typically the filename or a path-derived module identifier.

use crate::entry::Entry;
use crate::linters::{CommandLinter, Linters, Spec, into_entries};

use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio_stream::Stream;

pub struct LuaLuau(CommandLinter);

impl LuaLuau {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        Ok(Self(CommandLinter::new(
            linters,
            Spec {
                name: "luau-analyze",
                findings_on_stderr: true,
                parse: |f, l| into_entries(f, l, Self::parse_line),
                ..Default::default()
            },
            filename,
        )?))
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        let line = line.trim();
        // Format: name(line,col): Type: message
        let paren_open = line.find('(')?;
        let rest = &line[paren_open + 1..];
        let paren_close = rest.find(')')?;
        let loc = &rest[..paren_close];
        let after_paren = &rest[paren_close + 1..];
        // after_paren starts with ": "
        let msg_part = after_paren.strip_prefix(": ")?;
        // msg_part is "Type: message"
        let (_type_name, msg) = msg_part.split_once(": ")?;
        let mut loc_parts = loc.splitn(2, ',');
        let line_num: u32 = loc_parts.next()?.parse().ok()?;
        let col_num: u32 = loc_parts.next()?.parse().ok()?;
        Some(Entry::new_line_col(filename, "luau-analyze", msg, line_num, col_num).unwrap())
    }
}

linter_stream!(LuaLuau);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_type_error() {
        let entry = LuaLuau::parse_line(
            Path::new("foo.luau"),
            "foo.luau(3,7): TypeError: Type 'nil' could be converted into type 'string'",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "foo.luau:3: [luau-analyze] Type 'nil' could be converted into type 'string'"
        );
    }

    #[test]
    fn parse_line_syntax_error() {
        let entry = LuaLuau::parse_line(
            Path::new("test.luau"),
            "test.luau(1,1): SyntaxError: Expected identifier when parsing function, got '='",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.luau:1: [luau-analyze] Expected identifier when parsing function, got '='"
        );
    }

    #[test]
    fn parse_line_lint_warning() {
        let entry = LuaLuau::parse_line(
            Path::new("bar.luau"),
            "bar.luau(10,5): Warning: Unused variable 'x'",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "bar.luau:10: [luau-analyze] Unused variable 'x'"
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(LuaLuau::parse_line(Path::new("foo.luau"), "").is_none());
    }

    #[test]
    fn parse_line_no_parens() {
        assert!(LuaLuau::parse_line(Path::new("foo.luau"), "no parens here").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(LuaLuau::parse_line(Path::new("foo.luau"), "foo.luau(x,y): Error: msg").is_none());
    }

    #[test]
    fn parse_line_no_message() {
        assert!(LuaLuau::parse_line(Path::new("foo.luau"), "foo.luau(1,2): Error").is_none());
    }
}
