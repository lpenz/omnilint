// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::PathBuf;

use crate::cli::LinterMode;

/// Errors that can occur while loading the configuration.
#[derive(Debug)]
pub(crate) enum ConfigError {
    /// The config file could not be read.
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    /// The config file is not valid TOML.
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Read { path, source } => {
                write!(f, "cannot read config file {}: {source}", path.display())
            }
            ConfigError::Parse { path, source } => {
                write!(f, "invalid config file {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Read { source, .. } => Some(source),
            ConfigError::Parse { source, .. } => Some(source),
        }
    }
}

/// Global configuration options.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct GlobalConfig {
    /// Default mode for linters that are not found on the `PATH`.
    pub(crate) default_linter_mode: LinterMode,
}

/// Per-linter configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct LinterConfig {
    /// Controls what happens when this linter binary is not found.
    pub(crate) mode: LinterMode,
    /// Custom path to the linter binary, overriding PATH lookup.
    pub(crate) path: Option<String>,
}

impl Default for LinterConfig {
    fn default() -> Self {
        Self {
            mode: LinterMode::Wanted,
            path: None,
        }
    }
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
        if other.global.default_linter_mode != LinterMode::default() {
            self.global.default_linter_mode = other.global.default_linter_mode;
        }
        for (name, linter) in &other.linters {
            self.linters
                .entry(name.clone())
                .and_modify(|existing| {
                    existing.mode = linter.mode;
                    if linter.path.is_some() {
                        existing.path = linter.path.clone();
                    }
                })
                .or_insert_with(|| linter.clone());
        }
    }

    /// Loads the config by merging the config files in order.
    ///
    /// If `path` is `Some`, only that single file is loaded, overriding any
    /// automatic config discovery. Otherwise, the following sources are
    /// merged in order:
    /// 1. The `OMNILINT_CONFIG` environment variable
    /// 2. `/etc/omnilint.toml`
    /// 3. `~/.config/omnilint/omnilint.toml`
    /// 4. `<cwd>/omnilint.toml`
    pub(crate) fn load(path: Option<&std::path::Path>) -> Result<Self, ConfigError> {
        let mut config = Config::default();

        if let Some(path) = path {
            config.merge(&Config::from_file(path)?);
            return Ok(config);
        }

        // 1. OMNILINT_CONFIG environment variable
        if let Ok(path) = std::env::var("OMNILINT_CONFIG") {
            config.merge(&Config::from_file(std::path::Path::new(&path))?);
        }

        // 2. System-wide config
        if let Ok(content) = fs::read_to_string("/etc/omnilint.toml") {
            config.merge(&Config::parse(
                &content,
                std::path::Path::new("/etc/omnilint.toml"),
            )?);
        }

        // 3. User config
        if let Some(home) = dirs() {
            let path = home.join(".config/omnilint/omnilint.toml");
            if let Ok(content) = fs::read_to_string(&path) {
                config.merge(&Config::parse(&content, &path)?);
            }
        }

        // 4. Project config
        if let Ok(content) = fs::read_to_string("omnilint.toml") {
            config.merge(&Config::parse(
                &content,
                std::path::Path::new("omnilint.toml"),
            )?);
        }

        Ok(config)
    }

    /// Loads a single config file from the given path.
    fn from_file(path: &std::path::Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path).map_err(|e| ConfigError::Read {
            path: path.to_path_buf(),
            source: e,
        })?;
        Config::parse(&content, path)
    }

    /// Parses config contents, keeping the file path for error reporting.
    fn parse(content: &str, path: &std::path::Path) -> Result<Self, ConfigError> {
        let config: Config = toml::from_str(content).map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source,
        })?;
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
        assert_eq!(config.global.default_linter_mode, LinterMode::Wanted);
        assert!(config.linters.is_empty());
    }

    #[test]
    fn parse_global_only() {
        let config: Config =
            toml::from_str("[global]\ndefault_linter_mode = \"optional\"\n").unwrap();
        assert_eq!(config.global.default_linter_mode, LinterMode::Optional);
    }

    #[test]
    fn parse_linters_only() {
        let config: Config = toml::from_str("[linters.flake8]\nmode = \"disabled\"\n").unwrap();
        assert_eq!(config.linters["flake8"].mode, LinterMode::Disabled);
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
    fn merge_default_linter_mode() {
        let mut base = Config::default();
        let mut overlay = Config::default();
        overlay.global.default_linter_mode = LinterMode::Optional;
        base.merge(&overlay);
        assert_eq!(base.global.default_linter_mode, LinterMode::Optional);
    }

    #[test]
    fn merge_mode_overrides() {
        let mut base = Config::default();
        base.linters.insert(
            "flake8".to_string(),
            LinterConfig {
                mode: LinterMode::Disabled,
                ..Default::default()
            },
        );
        let mut overlay = Config::default();
        overlay.linters.insert(
            "ruff".to_string(),
            LinterConfig {
                mode: LinterMode::Disabled,
                ..Default::default()
            },
        );
        base.merge(&overlay);
        assert_eq!(base.linters["flake8"].mode, LinterMode::Disabled);
        assert_eq!(base.linters["ruff"].mode, LinterMode::Disabled);
    }

    #[test]
    fn merge_mode_last_wins() {
        let mut base = Config::default();
        base.global.default_linter_mode = LinterMode::Optional;
        base.linters.insert(
            "ruff".to_string(),
            LinterConfig {
                mode: LinterMode::Required,
                ..Default::default()
            },
        );
        let mut overlay = Config::default();
        overlay.linters.insert(
            "ruff".to_string(),
            LinterConfig {
                mode: LinterMode::Disabled,
                ..Default::default()
            },
        );
        base.merge(&overlay);
        assert_eq!(base.global.default_linter_mode, LinterMode::Optional);
        assert_eq!(base.linters["ruff"].mode, LinterMode::Disabled);
    }
}
