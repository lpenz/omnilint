// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of JavaScript and TypeScript files,
//! backed by oxlint and eslint.
//!
//! Requires `oxlint` and `eslint` to be available on the `PATH`, as well as
//! an `eslint.config.mjs` file in the fixtures directory.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean_js() {
    assert_eq!(common::run_clean(&["js-clean.js"]), "");
}

#[test]
fn clean_jsx() {
    assert_eq!(common::run_clean(&["js-clean.jsx"]), "");
}

#[test]
fn clean_mjs() {
    assert_eq!(common::run_clean(&["js-clean.mjs"]), "");
}

#[test]
fn clean_cjs() {
    assert_eq!(common::run_clean(&["js-clean.cjs"]), "");
}

#[test]
fn clean_ts() {
    assert_eq!(common::run_clean(&["ts-clean.ts"]), "");
}

#[test]
fn clean_tsx() {
    assert_eq!(common::run_clean(&["ts-clean.tsx"]), "");
}

#[test]
fn clean_mts() {
    assert_eq!(common::run_clean(&["ts-clean.mts"]), "");
}

#[test]
fn clean_cts() {
    assert_eq!(common::run_clean(&["ts-clean.cts"]), "");
}

#[test]
fn dirty_js() {
    assert_eq!(
        common::run(&["js-dirty.js"]),
        "Error: lint findings were emitted\n\
         js-dirty.js:1: [eslint] Parsing error: Unexpected keyword 'debugger'\n\
         js-dirty.js:1: [oxlint] Identifier expected. 'debugger' is a reserved word that cannot be used here. [Error]\n"
    );
}

#[test]
fn dirty_jsx() {
    assert_eq!(
        common::run(&["js-dirty.jsx"]),
        "Error: lint findings were emitted\n\
         js-dirty.jsx:1: [oxlint] Identifier expected. 'debugger' is a reserved word that cannot be used here. [Error]\n"
    );
}

#[test]
fn dirty_mjs() {
    assert_eq!(
        common::run(&["js-dirty.mjs"]),
        "Error: lint findings were emitted\n\
         js-dirty.mjs:1: [eslint] Parsing error: Unexpected keyword 'debugger'\n\
         js-dirty.mjs:1: [oxlint] Identifier expected. 'debugger' is a reserved word that cannot be used here. [Error]\n"
    );
}

#[test]
fn dirty_cjs() {
    assert_eq!(
        common::run(&["js-dirty.cjs"]),
        "Error: lint findings were emitted\n\
         js-dirty.cjs:1: [eslint] Parsing error: Unexpected keyword 'debugger'\n\
         js-dirty.cjs:1: [oxlint] Identifier expected. 'debugger' is a reserved word that cannot be used here. [Error]\n"
    );
}

#[test]
fn dirty_ts() {
    assert_eq!(
        common::run(&["ts-dirty.ts"]),
        "Error: lint findings were emitted\n\
         ts-dirty.ts:1: [oxlint] Identifier expected. 'debugger' is a reserved word that cannot be used here. [Error]\n"
    );
}

#[test]
fn dirty_tsx() {
    assert_eq!(
        common::run(&["ts-dirty.tsx"]),
        "Error: lint findings were emitted\n\
         ts-dirty.tsx:1: [oxlint] Identifier expected. 'debugger' is a reserved word that cannot be used here. [Error]\n"
    );
}

#[test]
fn dirty_mts() {
    assert_eq!(
        common::run(&["ts-dirty.mts"]),
        "Error: lint findings were emitted\n\
         ts-dirty.mts:1: [oxlint] Identifier expected. 'debugger' is a reserved word that cannot be used here. [Error]\n"
    );
}

#[test]
fn dirty_cts() {
    assert_eq!(
        common::run(&["ts-dirty.cts"]),
        "Error: lint findings were emitted\n\
         ts-dirty.cts:1: [oxlint] Identifier expected. 'debugger' is a reserved word that cannot be used here. [Error]\n"
    );
}
