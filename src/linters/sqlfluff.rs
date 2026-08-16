// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [sqlfluff](https://sqlfluff.com/) SQL linter wrapper.
//!
//! sqlfluff checks SQL files for style and correctness. It is run once per
//! file with the `ansi` dialect and the GitHub Actions annotation format, so
//! that each finding is printed on a single stdout line, analysing a single
//! file and creating no build artifacts.
//!
//! ## Output format
//!
//! Each finding emitted by sqlfluff on stdout has the form:
//!
//! ```text
//! ::warning title=SQLFluff,file=<filename>,line=<line>,col=<col>,...::<message>
//! ```
//!
//! For example:
//!
//! ```text
//! ::warning title=SQLFluff,file=foo.sql,line=1,col=1,endLine=1,endColumn=18::AM04: Query produces an unknown number of result columns. [ambiguous.column_count]
//! ```
//!
//! The annotations are wrapped in `::group::` and `::endgroup::` lines that
//! do not match the format above and are skipped by the parser.

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct SqlSqlfluff {
    filename: PathBuf,
    inner: Linter,
}

impl SqlSqlfluff {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let mut cmd = Command::new("sqlfluff");
        cmd.arg("lint");
        cmd.arg("--dialect");
        cmd.arg("ansi");
        cmd.arg("--format");
        cmd.arg("github-annotation-native");
        cmd.arg(filename);
        let inner = linters.spawn("sqlfluff", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }
        let rest = line.strip_prefix("::")?;
        let (attrs, msg) = rest.split_once("::")?;
        let line_num = attr_value(attrs, "line=")?;
        let col_num = attr_value(attrs, "col=")?;
        Some(Entry::new_line_col(filename, "sqlfluff", msg.trim(), line_num, col_num).unwrap())
    }
}

/// Parses the numeric value of the `key` attribute in a GitHub Actions
/// annotation attribute list.
fn attr_value(attrs: &str, key: &str) -> Option<u32> {
    let start = attrs.find(key)? + key.len();
    let end = attrs[start..]
        .find(',')
        .map(|i| start + i)
        .unwrap_or(attrs.len());
    attrs[start..end].parse().ok()
}

impl Stream for SqlSqlfluff {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "sqlfluff",
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
        let entry = SqlSqlfluff::parse_line(
            Path::new("foo.sql"),
            "::warning title=SQLFluff,file=foo.sql,line=1,col=1,endLine=1,endColumn=18::AM04: Query produces an unknown number of result columns. [ambiguous.column_count]",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "foo.sql:1: [sqlfluff] AM04: Query produces an unknown number of result columns. [ambiguous.column_count]"
        );
    }

    #[test]
    fn parse_line_skips_group() {
        assert!(SqlSqlfluff::parse_line(Path::new("foo.sql"), "::group::foo.sql").is_none());
        assert!(SqlSqlfluff::parse_line(Path::new("foo.sql"), "::endgroup::").is_none());
    }

    #[test]
    fn parse_line_empty() {
        assert!(SqlSqlfluff::parse_line(Path::new("foo.sql"), "").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(
            SqlSqlfluff::parse_line(
                Path::new("foo.sql"),
                "::warning title=SQLFluff,file=foo.sql,line=x,col=y::msg"
            )
            .is_none()
        );
    }

    #[test]
    fn attr_value_standard() {
        assert_eq!(
            attr_value(
                "title=SQLFluff,file=foo.sql,line=1,col=2,endLine=1",
                "line="
            ),
            Some(1)
        );
        assert_eq!(
            attr_value("title=SQLFluff,file=foo.sql,line=1", "col="),
            None
        );
    }
}
