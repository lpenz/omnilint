// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Default)]
pub enum Filetype {
    #[default]
    Unknown,
    Python,
    Yaml,
    Shell,
    Lua,
    Perl,
    Clojure,
    Dockerfile,
    Kotlin,
    Swift,
    Sql,
    Markdown,
    Xml,
}

impl Filetype {
    pub fn detect(path: &Path) -> Filetype {
        let ext = path.extension().and_then(|e| e.to_str());
        match ext {
            Some("yaml" | "yml") => Filetype::Yaml,
            Some("py") => Filetype::Python,
            Some("sh" | "bash" | "dash" | "ksh") => Filetype::Shell,
            Some("lua") => Filetype::Lua,
            Some("pl" | "pm") => Filetype::Perl,
            Some("clj" | "cljs" | "cljc" | "edn") => Filetype::Clojure,
            Some("kt" | "kts") => Filetype::Kotlin,
            Some("swift") => Filetype::Swift,
            Some("sql") => Filetype::Sql,
            Some("md" | "markdown") => Filetype::Markdown,
            Some("xml") => Filetype::Xml,
            _ => detect_filename_or_shebang(path),
        }
    }
}

/// Detects the file type from the filename for files without a distinguishing
/// extension (currently just Dockerfiles), falling back to shebang detection.
fn detect_filename_or_shebang(path: &Path) -> Filetype {
    if let Some(name) = path.file_name().and_then(|n| n.to_str())
        && is_dockerfile_name(name)
    {
        return Filetype::Dockerfile;
    }
    detect_shebang(path)
}

/// Returns true if `name` is a Dockerfile filename: `Dockerfile`,
/// `Dockerfile.*`, `Containerfile`, `Containerfile.*` or any `*.dockerfile`
/// / `*.containerfile`.
fn is_dockerfile_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower == "dockerfile"
        || lower == "containerfile"
        || lower.starts_with("dockerfile.")
        || lower.starts_with("containerfile.")
        || lower.ends_with(".dockerfile")
        || lower.ends_with(".containerfile")
}

fn detect_shebang(path: &Path) -> Filetype {
    let mut first_line = String::new();
    let result = fs::File::open(path).and_then(|f| {
        let mut bufreader = io::BufReader::new(f);
        io::BufRead::read_line(&mut bufreader, &mut first_line)
    });
    if result.is_err() {
        return Filetype::Unknown;
    }
    let shebang = first_line.strip_prefix("#!").unwrap_or_default();
    let mut parts = shebang.split_whitespace();
    let mut interp_name = Path::new(parts.next().unwrap_or_default())
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_string();
    if interp_name == "env" {
        interp_name = parts.next().unwrap_or_default().to_string();
    }
    match interp_name.as_str() {
        "python" | "python2" | "python3" => Filetype::Python,
        "sh" | "bash" | "dash" | "ksh" => Filetype::Shell,
        _ => Filetype::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_yaml() {
        assert_eq!(Filetype::detect(Path::new("foo.yaml")), Filetype::Yaml);
        assert_eq!(Filetype::detect(Path::new("foo.yml")), Filetype::Yaml);
    }

    #[test]
    fn detect_clojure() {
        assert_eq!(Filetype::detect(Path::new("foo.clj")), Filetype::Clojure);
        assert_eq!(Filetype::detect(Path::new("foo.cljs")), Filetype::Clojure);
        assert_eq!(Filetype::detect(Path::new("foo.cljc")), Filetype::Clojure);
        assert_eq!(Filetype::detect(Path::new("foo.edn")), Filetype::Clojure);
    }

    #[test]
    fn detect_dockerfile() {
        assert_eq!(
            Filetype::detect(Path::new("Dockerfile")),
            Filetype::Dockerfile
        );
        assert_eq!(
            Filetype::detect(Path::new("Dockerfile.dev")),
            Filetype::Dockerfile
        );
        assert_eq!(
            Filetype::detect(Path::new("containerfile")),
            Filetype::Dockerfile
        );
        assert_eq!(
            Filetype::detect(Path::new("app.dockerfile")),
            Filetype::Dockerfile
        );
    }

    #[test]
    fn detect_kotlin() {
        assert_eq!(Filetype::detect(Path::new("foo.kt")), Filetype::Kotlin);
        assert_eq!(Filetype::detect(Path::new("foo.kts")), Filetype::Kotlin);
    }

    #[test]
    fn detect_swift() {
        assert_eq!(Filetype::detect(Path::new("foo.swift")), Filetype::Swift);
    }

    #[test]
    fn detect_sql() {
        assert_eq!(Filetype::detect(Path::new("foo.sql")), Filetype::Sql);
    }

    #[test]
    fn detect_markdown() {
        assert_eq!(Filetype::detect(Path::new("foo.md")), Filetype::Markdown);
        assert_eq!(
            Filetype::detect(Path::new("foo.markdown")),
            Filetype::Markdown
        );
    }

    #[test]
    fn detect_xml() {
        assert_eq!(Filetype::detect(Path::new("foo.xml")), Filetype::Xml);
    }

    #[test]
    fn detect_python() {
        assert_eq!(Filetype::detect(Path::new("foo.py")), Filetype::Python);
    }

    #[test]
    fn detect_shell() {
        assert_eq!(Filetype::detect(Path::new("foo.sh")), Filetype::Shell);
        assert_eq!(Filetype::detect(Path::new("foo.bash")), Filetype::Shell);
        assert_eq!(Filetype::detect(Path::new("foo.dash")), Filetype::Shell);
        assert_eq!(Filetype::detect(Path::new("foo.ksh")), Filetype::Shell);
    }

    #[test]
    fn detect_lua() {
        assert_eq!(Filetype::detect(Path::new("foo.lua")), Filetype::Lua);
    }

    #[test]
    fn detect_unknown() {
        assert_eq!(Filetype::detect(Path::new("foo.txt")), Filetype::Unknown);
        assert_eq!(Filetype::detect(Path::new(".py")), Filetype::Unknown);
        assert_eq!(
            Filetype::detect(Path::new("no-such-file")),
            Filetype::Unknown
        );
    }

    #[test]
    fn detect_shebang() {
        let dir = std::env::temp_dir().join("omnilint-filetype-test");
        std::fs::create_dir_all(&dir).unwrap();

        let python = dir.join("tool");
        std::fs::write(&python, "#!/usr/bin/env python3\nprint('hi')\n").unwrap();
        assert_eq!(Filetype::detect(&python), Filetype::Python);

        let python_direct = dir.join("tool-direct");
        std::fs::write(&python_direct, "#!/usr/bin/python3\n").unwrap();
        assert_eq!(Filetype::detect(&python_direct), Filetype::Python);

        let shell = dir.join("script");
        std::fs::write(&shell, "#!/bin/bash\necho hi\n").unwrap();
        assert_eq!(Filetype::detect(&shell), Filetype::Shell);

        let plain = dir.join("data");
        std::fs::write(&plain, "just some text\n").unwrap();
        assert_eq!(Filetype::detect(&plain), Filetype::Unknown);

        let empty = dir.join("empty");
        std::fs::write(&empty, "").unwrap();
        assert_eq!(Filetype::detect(&empty), Filetype::Unknown);

        std::fs::remove_dir_all(&dir).unwrap();
    }
}
