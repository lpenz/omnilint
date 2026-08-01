// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of Python files, backed by flake8.
//!
//! Requires `flake8` to be available on the `PATH`.

mod common;

#[test]
fn clean() {
    assert_eq!(common::run(&["python-clean.py"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["python-dirty.py"]),
        "python-dirty.py:1:1: F401 'os' imported but unused\n\
         python-dirty.py:3:1: E302 expected 2 blank lines, found 1\n\
         python-dirty.py:4:5: F841 local variable 'unused' is assigned to but never used\n\
         python-dirty.py:7:1: E305 expected 2 blank lines after class or function definition, found 1\n\
         python-dirty.py:8:5: F821 undefined name 'undefined_name'\n"
    );
}
