// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of CSS, SCSS, Sass and Less files,
//! backed by stylelint.
//!
//! Requires `stylelint` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run_clean(&["css-clean.css"]), "");
}

#[test]
fn clean_scss() {
    assert_eq!(common::run_clean(&["css-clean.scss"]), "");
}

#[test]
fn clean_sass() {
    assert_eq!(common::run_clean(&["css-clean.sass"]), "");
}

#[test]
fn clean_less() {
    assert_eq!(common::run_clean(&["css-clean.less"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["css-dirty.css"]),
        "Error: lint findings were emitted\n\
         css-dirty.css:1: [stylelint] Empty block (block-no-empty) [error]\n"
    );
}

#[test]
fn dirty_scss() {
    assert_eq!(
        common::run(&["css-dirty.scss"]),
        "Error: lint findings were emitted\n\
         css-dirty.scss:1: [stylelint] Empty block (block-no-empty) [error]\n"
    );
}

#[test]
fn dirty_sass() {
    assert_eq!(
        common::run(&["css-dirty.sass"]),
        "Error: lint findings were emitted\n\
         css-dirty.sass:1: [stylelint] Empty block (block-no-empty) [error]\n"
    );
}

#[test]
fn dirty_less() {
    assert_eq!(
        common::run(&["css-dirty.less"]),
        "Error: lint findings were emitted\n\
         css-dirty.less:1: [stylelint] Empty block (block-no-empty) [error]\n"
    );
}
