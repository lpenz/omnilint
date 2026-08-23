// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Integration tests for the analysis of PHP files, backed by `php -l`.
//!
//! Requires `php` to be available on the `PATH`.
//!
//! Only runs when the `test-linter-tools` feature is enabled.

#![cfg(feature = "test-linter-tools")]

mod common;

#[test]
fn clean() {
    assert_eq!(common::run_clean(&["php-clean.php"]), "");
}

#[test]
fn dirty() {
    assert_eq!(
        common::run(&["php-dirty.php"]),
        "php-dirty.php:1: [phpcs] Header blocks must be separated by a single blank line\n\
         php-dirty.php:2: [php] syntax error, unexpected token \";\"\n"
    );
}

#[test]
fn md_dirty() {
    assert_eq!(
        common::run(&["php-md-dirty.php"]),
        "php-md-dirty.php:7: [phpmd] ShortVariable: Avoid variables with short names like $x. Configured minimum length is 3.\n\
         php-md-dirty.php:9: [phpmd] UnusedLocalVariable: Avoid unused local variables such as '$unused'.\n"
    );
}
