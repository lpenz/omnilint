// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the behavior when the linter tools are not found on
//! the `PATH`: instead of aborting, an entry reporting the missing linter is
//! produced for each file.

mod common;

#[test]
fn linters_not_found() {
    assert_eq!(
        common::run_without_linters(&[
            "clj-clean.clj",
            "python-clean.py",
            "shell-clean.sh",
            "yaml-clean.yaml",
        ]),
        "clj-clean.clj: [clj-kondo] linter not found\n\
         python-clean.py: [flake8] linter not found\n\
         python-clean.py: [ruff] linter not found\n\
         shell-clean.sh: [shellcheck] linter not found\n\
         yaml-clean.yaml: [yamllint] linter not found\n"
    );
}
