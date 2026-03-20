use crate::rules::{self, Rule};
use serde::Deserialize;
use sqruff_parser::dialect::Dialect;
use std::path::Path;

/// Configuration loaded from `.sqruff.toml`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub dialect: String,
    pub rules: RulesConfig,
    pub format: FormatConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct RulesConfig {
    /// Disable specific rules by code (e.g. ["SQ001", "SQ003"]).
    pub disable: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct FormatConfig {
    /// Indentation string (default: 2 spaces).
    pub indent: String,
    /// Uppercase keywords (default: true).
    pub uppercase_keywords: bool,
    /// Max line width (0 = unlimited).
    pub max_line_width: usize,
    /// Trailing semicolon (default: true).
    pub trailing_semicolon: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dialect: "generic".to_string(),
            rules: RulesConfig::default(),
            format: FormatConfig::default(),
        }
    }
}

impl Default for RulesConfig {
    fn default() -> Self {
        Self {
            disable: Vec::new(),
        }
    }
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            indent: "  ".to_string(),
            uppercase_keywords: true,
            max_line_width: 120,
            trailing_semicolon: true,
        }
    }
}

impl Config {
    /// Load config from a `.sqruff.toml` file, falling back to defaults.
    pub fn load(path: Option<&Path>) -> Self {
        if let Some(p) = path {
            if let Ok(contents) = std::fs::read_to_string(p) {
                if let Ok(config) = toml::from_str(&contents) {
                    return config;
                }
            }
        }
        // Try current directory
        if let Ok(contents) = std::fs::read_to_string(".sqruff.toml") {
            if let Ok(config) = toml::from_str(&contents) {
                return config;
            }
        }
        Config::default()
    }

    pub fn dialect(&self) -> Dialect {
        Dialect::from_str_loose(&self.dialect)
    }

    /// Get all active rules (all rules minus disabled ones).
    pub fn active_rules(&self) -> Vec<Box<dyn Rule>> {
        let all = rules::all_rules();
        if self.rules.disable.is_empty() {
            return all;
        }
        all.into_iter()
            .filter(|r| !self.rules.disable.contains(&r.code().to_string()))
            .collect()
    }
}
