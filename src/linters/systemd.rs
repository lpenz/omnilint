// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [systemd-analyze verify](https://www.freedesktop.org/software/systemd/man/latest/systemd-analyze.html)
//! systemd unit linter wrapper.
//!
//! `systemd-analyze verify` checks systemd unit files for errors such as
//! unknown keys, invalid values and broken dependencies. Findings for the
//! given file are parsed into [`Entry`] values; findings about dependency
//! units (which verify also reports) are discarded by matching the file
//! name suffix.
//!
//! ## Output format
//!
//! Each finding is a single line on stderr, with the form:
//!
//! ```text
//! <absolute filename>:<line>: <message>
//! ```
//!
//! For example:
//!
//! ```text
//! /home/user/foo.service:5: Unknown key 'Foo' in section [Service], ignoring.
//! ```

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct SystemdAnalyze {
    filename: PathBuf,
    inner: Linter,
}

impl SystemdAnalyze {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("systemd-analyze");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("verify");
        cmd.arg(filename);
        let inner = linters.spawn("systemd-analyze", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        let line = line.trim();
        let base = filename.file_name()?.to_str()?;
        let parts: Vec<&str> = line.splitn(3, ':').collect();
        if parts.len() < 3 {
            return None;
        }
        // Discard findings about dependency units, keeping only the ones
        // about the file being analysed
        if !parts[0].ends_with(base) {
            return None;
        }
        let num: u32 = parts[1].trim().parse().ok()?;
        let msg = parts[2].trim();
        if msg.is_empty() {
            return None;
        }
        Some(Entry::new_line(filename, "systemd-analyze", msg, num).unwrap())
    }
}

impl Stream for SystemdAnalyze {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "systemd-analyze",
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
        let entry = SystemdAnalyze::parse_line(
            Path::new("test.service"),
            "/tmp/test.service:5: Unknown key 'Foo' in section [Service], ignoring.",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "test.service:5: [systemd-analyze] Unknown key 'Foo' in section [Service], ignoring."
        );
    }

    #[test]
    fn parse_line_discards_other_units() {
        assert!(
            SystemdAnalyze::parse_line(
                Path::new("test.service"),
                "/usr/lib/systemd/system/xfs_scrub_all.service:26: Support for option CPUAccounting= has been removed and it is ignored"
            )
            .is_none()
        );
    }

    #[test]
    fn parse_line_discards_unnumbered_lines() {
        assert!(
            SystemdAnalyze::parse_line(
                Path::new("test.service"),
                "Binding to IPv6 address not available since kernel does not support IPv6."
            )
            .is_none()
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(SystemdAnalyze::parse_line(Path::new("test.service"), "").is_none());
    }

    #[test]
    fn parse_line_too_few_parts() {
        assert!(SystemdAnalyze::parse_line(Path::new("test.service"), "no colons").is_none());
    }

    #[test]
    fn parse_line_non_numeric() {
        assert!(
            SystemdAnalyze::parse_line(Path::new("test.service"), "/tmp/test.service:x: msg")
                .is_none()
        );
    }
}
