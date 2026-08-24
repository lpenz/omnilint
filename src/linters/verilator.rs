// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [verilator](https://www.veripool.org/verilator/) Verilog/SystemVerilog
//! linter wrapper.
//!
//! Verilator is run once per file with `--lint-only`, so that it only checks
//! the file for lint issues and creates no build artifacts.
//!
//! ## Output format
//!
//! Each finding emitted by verilator on stderr has the form:
//!
//! ```text
//! %<severity>: <filename>:<line>:<col>: <message>
//! ```
//!
//! For example:
//!
//! ```text
//! %Warning-WIDTHTRUNC: verilog-dirty.sv:5:16: Operator ASSIGNW expects 4 bits on the Assign RHS, but Assign RHS's CONST '8'hff' generates 8 bits.
//! ```
//!
//! Verilator also prints source context lines, `...` notes and a final
//! `%Error: Exiting due to ...` summary; those lines do not have a location in
//! the format above and are skipped by the parser, as are findings about other
//! files such as includes. The severity prefix is discarded, keeping the
//! message. The banner printed on stdout is ignored by reading the findings
//! from stderr only.

use crate::entry::Entry;
use crate::linters::{Linter, Linters};

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio::process::Command;
use tokio_stream::Stream;

pub struct VerilogVerilator {
    filename: PathBuf,
    inner: Linter,
}

impl VerilogVerilator {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        let executable = linters.executable("verilator");
        let mut cmd = Command::new(executable.as_ref());
        cmd.arg("--lint-only");
        cmd.arg(filename);
        let inner = linters.spawn("verilator", cmd)?;
        Ok(Self {
            filename: filename.to_path_buf(),
            inner,
        })
    }

    fn parse_line(filename: &Path, line: &str) -> Option<Entry> {
        let line = line.trim();
        let rest = line.strip_prefix('%')?;
        let (_severity, loc_msg) = rest.split_once(": ")?;
        let base = filename.file_name()?.to_str()?;
        let fields: Vec<&str> = loc_msg.splitn(4, ':').collect();
        // Discard findings about other files, keeping only the ones about
        // the file being analysed
        if !fields.first()?.ends_with(base) {
            return None;
        }
        let line_num: u32 = fields.get(1)?.parse().ok()?;
        let entry = match fields.as_slice() {
            [_, _, msg] => Entry::new_line(filename, "verilator", msg.trim(), line_num),
            [_, _, col_num, msg] => Entry::new_line_col(
                filename,
                "verilator",
                msg.trim(),
                line_num,
                col_num.parse().ok()?,
            ),
            _ => return None,
        };
        Some(entry.unwrap())
    }
}

impl Stream for VerilogVerilator {
    type Item = Entry;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        crate::linters::poll_next(
            "verilator",
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
    fn parse_line_warning_with_col() {
        let entry = VerilogVerilator::parse_line(
            Path::new("foo.sv"),
            "%Warning-WIDTHTRUNC: foo.sv:5:16: Operator ASSIGNW expects 4 bits on the Assign RHS.",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "foo.sv:5: [verilator] Operator ASSIGNW expects 4 bits on the Assign RHS."
        );
    }

    #[test]
    fn parse_line_error_with_col() {
        let entry = VerilogVerilator::parse_line(
            Path::new("foo.v"),
            "%Error: foo.v:2:1: syntax error, unexpected endmodule, expecting '['",
        )
        .unwrap();
        assert_eq!(
            entry.to_string(),
            "foo.v:2: [verilator] syntax error, unexpected endmodule, expecting '['"
        );
    }

    #[test]
    fn parse_line_without_col() {
        let entry =
            VerilogVerilator::parse_line(Path::new("foo.v"), "%Error: foo.v:3: oops").unwrap();
        assert_eq!(entry.to_string(), "foo.v:3: [verilator] oops");
    }

    #[test]
    fn parse_line_discards_other_files() {
        assert!(
            VerilogVerilator::parse_line(
                Path::new("foo.v"),
                "%Warning-UNUSEDSIGNAL: bar.v:7:2: Signal has no used bits"
            )
            .is_none()
        );
    }

    #[test]
    fn parse_line_skips_summary_and_notes() {
        assert!(
            VerilogVerilator::parse_line(Path::new("foo.v"), "%Error: Exiting due to 1 warning(s)")
                .is_none()
        );
        assert!(
            VerilogVerilator::parse_line(Path::new("foo.v"), ": ... note: In instance 't'")
                .is_none()
        );
        assert!(
            VerilogVerilator::parse_line(Path::new("foo.v"), "|    5 |   assign out;").is_none()
        );
    }

    #[test]
    fn parse_line_empty() {
        assert!(VerilogVerilator::parse_line(Path::new("foo.v"), "").is_none());
    }
}
