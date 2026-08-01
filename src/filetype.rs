// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::path::Path;

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Default)]
pub enum Filetype {
    #[default]
    Unknown,
    Python,
    Yaml,
    Shell,
}

impl Filetype {
    pub fn detect(path: &Path) -> Filetype {
        let ext = path.extension().and_then(|e| e.to_str());
        match ext {
            Some("yaml" | "yml") => Filetype::Yaml,
            Some("py") => Filetype::Python,
            Some("sh" | "bash" | "dash" | "ksh") => Filetype::Shell,
            _ => Filetype::Unknown,
        }
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
    fn detect_unknown() {
        assert_eq!(Filetype::detect(Path::new("foo.txt")), Filetype::Unknown);
        assert_eq!(Filetype::detect(Path::new("foo")), Filetype::Unknown);
        assert_eq!(Filetype::detect(Path::new(".py")), Filetype::Unknown);
    }
}
