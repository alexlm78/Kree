use std::collections::HashSet;
use std::fs;

pub struct IgnoreFilter {
    excluded: HashSet<String>,
    active: bool,
}

impl IgnoreFilter {
    pub fn new(active: bool) -> Self {
        if !active {
            return IgnoreFilter {
                excluded: HashSet::new(),
                active: false,
            };
        }

        let excluded = match fs::read_to_string(".kreeignore") {
            Ok(contents) => contents
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.trim().to_string())
                .collect(),
            Err(_) => HashSet::new(),
        };

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
