// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of Python files, backed by flake8 and
//! ruff.
//!
//! Requires `flake8` and `ruff` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(
        common::run(&["python-clean.py"]),
        "python-clean.py:1: [pylint] Constant name \"x\" doesn't conform to UPPER_CASE naming style (invalid-name)\n\
         python-clean.py:1: [pylint] Missing module docstring (missing-module-docstring)\n\
         python-clean.py:1: [pylint] Module name \"python-clean\" doesn't conform to snake_case naming style (invalid-name)\n"
    );
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["python-dirty.py"]),
        "python-dirty.py:1: [flake8] F401 'os' imported but unused\n\
         python-dirty.py:1: [pylint] Missing module docstring (missing-module-docstring)\n\
         python-dirty.py:1: [pylint] Module name \"python-dirty\" doesn't conform to snake_case naming style (invalid-name)\n\
         python-dirty.py:1: [pylint] Unused import os (unused-import)\n\
         python-dirty.py:1: [ruff] F401 [*] `os` imported but unused\n\
         python-dirty.py:1: [ruff] I001 [*] Import block is un-sorted or un-formatted\n\
         python-dirty.py:3: [flake8] E302 expected 2 blank lines, found 1\n\
         python-dirty.py:3: [pylint] Missing function or method docstring (missing-function-docstring)\n\
         python-dirty.py:4: [flake8] F841 local variable 'unused' is assigned to but never used\n\
         python-dirty.py:4: [pylint] Unused variable 'unused' (unused-variable)\n\
         python-dirty.py:4: [ruff] F841 Local variable `unused` is assigned to but never used\n\
         python-dirty.py:7: [flake8] E305 expected 2 blank lines after class or function definition, found 1\n\
         python-dirty.py:7: [pylint] Assigning result of a function call, where the function has no return (assignment-from-no-return)\n\
         python-dirty.py:7: [pylint] Constant name \"x\" doesn't conform to UPPER_CASE naming style (invalid-name)\n\
         python-dirty.py:8: [flake8] F821 undefined name 'undefined_name'\n\
         python-dirty.py:8: [pylint] Undefined variable 'undefined_name' (undefined-variable)\n\
         python-dirty.py:8: [ruff] F821 Undefined name `undefined_name`\n"
    );
}
