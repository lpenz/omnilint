// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [eslint](https://eslint.org/) JavaScript linter wrapper.
//!
//! eslint checks JavaScript source files for syntax errors, potential
//! problems and coding style issues. It is run with `--format=json` and
//! its output is parsed into [`Entry`] values.
//!
//! Note that eslint only runs on `.js` files: without a TypeScript parser
//! configured in the project, core ESLint cannot handle TypeScript syntax.
//!
//! ## Output format
//!
//! The JSON formatter emits a single line containing an array of results,
//! each with a list of messages:
//!
//! ```json
//! [{"filePath":"foo.js","messages":[{"ruleId":"eqeqeq","severity":2,"message":"Expected '==='.","line":2,"column":7}]}]
//! ```
//!
//! Each message is converted into one [`Entry`], with the rule identifier
//! appended to the message between parentheses. Messages without a line
//! number (e.g. the notice emitted for ignored files) are skipped.

use crate::entry::Entry;
use crate::linters::{CommandLinter, Linters, Spec};

use serde::Deserialize;
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use color_eyre::Result;
use tokio_stream::Stream;

#[derive(Deserialize)]
struct EsLintResult {
    #[serde(default)]
    messages: Vec<EsLintMessage>,
}

#[derive(Deserialize)]
struct EsLintMessage {
    line: Option<u32>,
    column: Option<u32>,
    message: String,
    #[serde(rename = "ruleId")]
    rule_id: Option<String>,
}

pub struct JsEslint(CommandLinter);

impl JsEslint {
    pub fn new(linters: &mut Linters, filename: &Path) -> Result<Self> {
        Ok(Self(CommandLinter::new(
            linters,
            Spec {
                name: "eslint",
                args: &["--format=json"],
                parse: parse_line,
                ..Default::default()
            },
            filename,
        )?))
    }
}

fn parse_line(filename: &Path, line: &str) -> Vec<Entry> {
    let Ok(results) = serde_json::from_str::<Vec<EsLintResult>>(line) else {
        return Vec::new();
    };
    let mut entries = Vec::new();
    for result in results {
        for message in result.messages {
            let Some(line_num) = message.line else {
                // Messages without a location, e.g. the notice emitted
                // for ignored files, are not findings.
                continue;
            };
            let msg = match &message.rule_id {
                Some(rule_id) => format!("{} ({rule_id})", message.message),
                None => message.message.clone(),
            };
            let entry = match message.column {
                Some(col_num) if col_num > 0 => {
                    Entry::new_line_col(filename, "eslint", &msg, line_num, col_num)
                }
                _ => Entry::new_line(filename, "eslint", &msg, line_num),
            };
            if let Ok(entry) = entry {
                entries.push(entry);
            }
        }
    }
    entries
}

linter_stream!(JsEslint);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_standard() {
        let entries = parse_line(
            Path::new("test.js"),
            r#"[{"filePath":"test.js","messages":[{"ruleId":"no-unused-vars","severity":2,"message":"'x' is assigned a value but never used.","line":1,"column":5}]}]"#,
        );
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].to_string(),
            "test.js:1: [eslint] 'x' is assigned a value but never used. (no-unused-vars)"
        );
    }

    #[test]
    fn parse_line_multiple_messages() {
        let entries = parse_line(
            Path::new("test.js"),
            r#"[{"filePath":"test.js","messages":[
                {"ruleId":"a-rule","severity":2,"message":"first","line":1,"column":1},
                {"ruleId":null,"severity":2,"message":"second","line":3,"column":9}
            ]}]"#,
        );
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].to_string(), "test.js:1: [eslint] first (a-rule)");
        assert_eq!(entries[1].to_string(), "test.js:3: [eslint] second");
    }

    #[test]
    fn parse_line_skips_messages_without_location() {
        let entries = parse_line(
            Path::new("test.ts"),
            r#"[{"filePath":"test.ts","messages":[{"ruleId":null,"severity":1,"message":"File ignored because no matching configuration was supplied."}]}]"#,
        );
        assert!(entries.is_empty());
    }

    #[test]
    fn parse_line_invalid_json() {
        assert!(parse_line(Path::new("test.js"), "not json").is_empty());
    }

    #[test]
    fn parse_line_empty() {
        assert!(parse_line(Path::new("test.js"), "").is_empty());
    }
}
