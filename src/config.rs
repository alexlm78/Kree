use std::collections::HashMap;
use std::fs;

use serde::Deserialize;

use crate::tree::SortMode;

/// Main configuration structure for Kree.
///
/// This struct is deserialized from the `~/.kreerc` file using TOML format.
/// It holds default settings, color mappings, ignore patterns, and icon mappings.
#[derive(Debug, Default, Deserialize)]
pub struct KreeConfig {
    /// Default values for command-line arguments.
    #[serde(default)]
    pub defaults: DefaultsConfig,
    /// Custom color mappings for file extensions.
    /// Key: file extension, Value: color name or hex code.
    #[serde(default)]
    pub colors: HashMap<String, String>,
    /// Configuration for ignored files and patterns.
    #[serde(default)]
    pub ignore: IgnoreConfig,
    /// Custom icon mappings for file extensions or filenames.
    /// Key: extension/filename, Value: Nerd Font icon character.
    #[serde(default)]
    pub icons: HashMap<String, String>,
}

/// Default configuration values that can be overridden by CLI arguments.
#[derive(Debug, Default, Deserialize)]
pub struct DefaultsConfig {
    /// Default recursion depth.
    pub depth: Option<u32>,
    /// Default sort mode ("name" or "kind").
    pub sort: Option<String>,
    /// Default setting for disabling colored output.
    pub no_color: Option<bool>,
    /// Default setting for showing hidden files.
    pub all: Option<bool>,
    /// Default setting for showing icons.
    pub icons: Option<bool>,
}

/// Configuration for ignore patterns.
#[derive(Debug, Default, Deserialize)]
pub struct IgnoreConfig {
    /// List of additional glob patterns to ignore.
    #[serde(default)]
    pub patterns: Vec<String>,
}

impl KreeConfig {
    /// Loads the configuration from `~/.kreerc`.
    ///
    /// If the file does not exist or cannot be read, returns a default configuration.
    /// If the file exists but is malformed, prints a warning and returns default configuration.
    pub fn load() -> Self {
        let Some(home) = dirs::home_dir() else {
            return Self::default();
        };

        let config_path = home.join(".kreerc");
        let contents = match fs::read_to_string(&config_path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };

        match toml::from_str(&contents) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Warning: malformed ~/.kreerc: {e}");
                Self::default()
            }
        }
    }

    /// Resolves the configured sort mode into a `SortMode` enum.
    ///
    /// Returns `None` if no sort mode is configured or if the configured string is invalid.
    pub fn sort_mode(&self) -> Option<SortMode> {
        self.defaults.sort.as_deref().and_then(|s| match s {
            "name" => Some(SortMode::Name),
            "kind" => Some(SortMode::Kind),
            other => {
                eprintln!("Warning: unknown sort mode '{other}' in ~/.kreerc, ignoring");
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_with_sort(sort: Option<&str>) -> KreeConfig {
        KreeConfig {
            defaults: DefaultsConfig {
                sort: sort.map(|s| s.to_string()),
                ..DefaultsConfig::default()
            },
            ..KreeConfig::default()
        }
    }

    #[test]
    fn sort_mode_name() {
        assert!(matches!(config_with_sort(Some("name")).sort_mode(), Some(SortMode::Name)));
    }

    #[test]
    fn sort_mode_kind() {
        assert!(matches!(config_with_sort(Some("kind")).sort_mode(), Some(SortMode::Kind)));
    }

    #[test]
    fn sort_mode_none() {
        assert!(config_with_sort(None).sort_mode().is_none());
    }

    #[test]
    fn sort_mode_invalid() {
        assert!(config_with_sort(Some("bogus")).sort_mode().is_none());
    }
}
