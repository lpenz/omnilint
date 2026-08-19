// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Global configuration options.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct GlobalConfig {
    /// Ignore linters that are not found on the `PATH`.
    pub(crate) ignore_missing_linters: bool,
}

/// Per-linter configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct LinterConfig {
    /// Whether this linter is enabled.
    #[serde(default = "default_true")]
    pub(crate) enabled: bool,
    /// Custom path to the linter binary, overriding PATH lookup.
    pub(crate) path: Option<String>,
}

impl Default for LinterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: None,
        }
    }
}

fn default_true() -> bool {
    true
}

/// The omnilint configuration, loaded from TOML files.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct Config {
    pub(crate) global: GlobalConfig,
    pub(crate) linters: HashMap<String, LinterConfig>,
}

impl Config {
    /// Merges another config into self: non-default values from `other`
    /// override values in self.
    fn merge(&mut self, other: &Config) {
        if other.global.ignore_missing_linters {
            self.global.ignore_missing_linters = true;
        }
        for (name, linter) in &other.linters {
            self.linters
                .entry(name.clone())
                .and_modify(|existing| {
                    if !linter.enabled {
                        existing.enabled = false;
                    }
                    if linter.path.is_some() {
                        existing.path = linter.path.clone();
                    }
                })
                .or_insert_with(|| linter.clone());
        }
    }

    /// Loads the config by merging the three config files in order:
    /// 1. `/etc/omnilint.toml`
    /// 2. `~/.config/omnilint/omnilint.toml`
    /// 3. `<cwd>/omnilint.toml`
    pub(crate) fn load() -> Result<Self, toml::de::Error> {
        let mut config = Config::default();

        // 1. OMNILINT_CONFIG environment variable
        if let Ok(path) = std::env::var("OMNILINT_CONFIG")
            && let Ok(content) = fs::read_to_string(&path)
        {
            config.merge(&toml::from_str(&content)?);
        }

        // 2. System-wide config
        if let Ok(content) = fs::read_to_string("/etc/omnilint.toml") {
            config.merge(&toml::from_str(&content)?);
        }

        // 3. User config
        if let Some(home) = dirs() {
            let path = home.join(".config/omnilint/omnilint.toml");
            if let Ok(content) = fs::read_to_string(path) {
                config.merge(&toml::from_str(&content)?);
            }
        }

        // 4. Project config
        if let Ok(content) = fs::read_to_string("omnilint.toml") {
            config.merge(&toml::from_str(&content)?);
        }

        Ok(config)
    }
}

fn dirs() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = Config::default();
        assert!(!config.global.ignore_missing_linters);
        assert!(config.linters.is_empty());
    }

    #[test]
    fn parse_global_only() {
        let config: Config = toml::from_str("[global]\nignore_missing_linters = true\n").unwrap();
        assert!(config.global.ignore_missing_linters);
    }

    #[test]
    fn parse_linters_only() {
        let config: Config = toml::from_str("[linters.flake8]\nenabled = false\n").unwrap();
        assert!(!config.linters["flake8"].enabled);
    }

    #[test]
    fn parse_linter_path() {
        let config: Config =
            toml::from_str("[linters.flake8]\npath = \"/usr/local/bin/flake8\"\n").unwrap();
        assert_eq!(
            config.linters["flake8"].path.as_deref(),
            Some("/usr/local/bin/flake8")
        );
    }

    #[test]
    fn merge_ignore_missing() {
        let mut base = Config::default();
        let mut overlay = Config::default();
        overlay.global.ignore_missing_linters = true;
        base.merge(&overlay);
        assert!(base.global.ignore_missing_linters);
    }

    #[test]
    fn merge_disabled_linters() {
        let mut base = Config::default();
        base.linters.insert(
            "flake8".to_string(),
            LinterConfig {
                enabled: false,
                ..Default::default()
            },
        );
        let mut overlay = Config::default();
        overlay.linters.insert(
            "ruff".to_string(),
            LinterConfig {
                enabled: false,
                ..Default::default()
            },
        );
        base.merge(&overlay);
        assert!(!base.linters["flake8"].enabled);
        assert!(!base.linters["ruff"].enabled);
    }

    #[test]
    fn merge_overrides() {
        let mut base = Config::default();
        base.global.ignore_missing_linters = true;
        base.linters.insert(
            "ruff".to_string(),
            LinterConfig {
                enabled: true,
                ..Default::default()
            },
        );
        let mut overlay = Config::default();
        overlay.linters.insert(
            "ruff".to_string(),
            LinterConfig {
                enabled: false,
                ..Default::default()
            },
        );
        base.merge(&overlay);
        assert!(base.global.ignore_missing_linters);
        assert!(!base.linters["ruff"].enabled);
    }
}
