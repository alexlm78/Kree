use std::collections::HashMap;
use std::fs;

use serde::Deserialize;

use crate::tree::SortMode;

#[derive(Debug, Default, Deserialize)]
pub struct KreeConfig {
    #[serde(default)]
    pub defaults: DefaultsConfig,
    #[serde(default)]
    pub colors: HashMap<String, String>,
    #[serde(default)]
    pub ignore: IgnoreConfig,
    #[serde(default)]
    pub icons: HashMap<String, String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct DefaultsConfig {
    pub depth: Option<u32>,
    pub sort: Option<String>,
    pub no_color: Option<bool>,
    pub all: Option<bool>,
    pub icons: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
pub struct IgnoreConfig {
    #[serde(default)]
    pub patterns: Vec<String>,
}

impl KreeConfig {
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
