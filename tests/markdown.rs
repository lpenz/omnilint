// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of Markdown files, backed by
//! `markdownlint-cli2` and `proselint`.
//!
//! Requires `markdownlint-cli2` and `proselint` to be available on the
//! `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run_clean(&["markdown-clean.md"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["markdown-prose-dirty.md"]),
        "Error: lint findings were emitted\n\
         markdown-prose-dirty.md:3: [proselint] uncomparables: Comparison of an uncomparable: 'very unique' is not comparable.\n\
         markdown-prose-dirty.md:3: [proselint] weasel_words.very: Substitute 'damn' every time you're inclined to write 'very'; your editor will delete it and the writing will be just as it should be.\n\
         markdown-prose-dirty.md:5: [proselint] misc.greylist: Use of 'utilize'. Do you know anyone who needs to utilize the word utilize?\n\
         markdown-prose-dirty.md:6: [proselint] redundancy.misc.after_the_deadline: Redundancy. Use 'every' instead of 'each and every'.\n"
    );
}
