use std::collections::HashSet;
use std::fs;

#[derive(Clone)]
pub struct IgnoreFilter {
    excluded: HashSet<String>,
    active: bool,
}

impl IgnoreFilter {
    pub fn new(active: bool, config_patterns: &[String]) -> Self {
        if !active {
            return IgnoreFilter {
                excluded: HashSet::new(),
                active: false,
            };
        }

        let mut excluded: HashSet<String> = match fs::read_to_string(".kreeignore") {
            Ok(contents) => contents
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.trim().to_string())
                .collect(),
            Err(_) => HashSet::new(),
        };

        for pattern in config_patterns {
            excluded.insert(pattern.clone());
        }

        IgnoreFilter {
            excluded,
            active: true,
        }
    }

    pub fn is_ignored(&self, filename: &str) -> bool {
        if !self.active {
            return false;
        }
        filename.starts_with('.') || self.excluded.contains(filename)
    }
}
