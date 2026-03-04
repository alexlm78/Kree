use std::fs;
use std::path::Path;

/// Represents a match found during fuzzy search.
pub struct SearchResult {
    /// Name of the matched file or directory.
    pub name: String,
    /// Full path string.
    pub path: String,
    /// Levenshtein distance score (lower is better).
    pub score: usize,
}

/// Calculates the Levenshtein distance between two strings.
///
/// The Levenshtein distance is the minimum number of single-character edits
/// (insertions, deletions or substitutions) required to change one word into the other.
pub fn levenshtein(s1: &str, s2: &str) -> usize {
    let s1: Vec<char> = s1.chars().collect();
    let s2: Vec<char> = s2.chars().collect();

    let m = s1.len();
    let n = s2.len();

    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0usize; n + 1];

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            if s1[i - 1] == s2[j - 1] {
                curr[j] = prev[j - 1];
            } else {
                curr[j] = 1 + prev[j].min(curr[j - 1]).min(prev[j - 1]);
            }
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}

/// Performs a recursive fuzzy search for files and directories matching a query.
///
/// # Arguments
///
/// * `root_path` - The root directory to start searching from.
/// * `query` - The search string.
/// * `max_depth` - Maximum recursion depth.
///
/// Returns a list of `SearchResult` sorted by score (ascending).
pub fn fuzzy_search(root_path: &Path, query: &str, max_depth: u32) -> Vec<SearchResult> {
    let mut results = Vec::new();
    search_recursive(root_path, query, max_depth, 0, &mut results);
    results.sort_by_key(|r| r.score);
    results
}

fn search_recursive(
    path: &Path,
    query: &str,
    max_depth: u32,
    current_depth: u32,
    results: &mut Vec<SearchResult>,
) {
    if current_depth > max_depth {
        return;
    }

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    let path_str = path.to_string_lossy().into_owned();

    let score_name = levenshtein(&name, query);
    let score_path = levenshtein(&path_str, query);
    let query_len = query.len();

    // Heuristic: only keep results where distance is reasonably small compared to query length
    if score_name * 100 <= 50 * query_len || score_path * 100 <= 50 * query_len {
        results.push(SearchResult {
            name,
            path: path_str,
            score: score_name.min(score_path),
        });
    }

    if !path.is_dir() {
        return;
    }

    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        search_recursive(&entry.path(), query, max_depth, current_depth + 1, results);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levenshtein_identical() {
        assert_eq!(levenshtein("kitten", "kitten"), 0);
    }

    #[test]
    fn levenshtein_single_sub() {
        assert_eq!(levenshtein("kitten", "sitten"), 1);
    }

    #[test]
    fn levenshtein_classic() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
    }

    #[test]
    fn levenshtein_empty_first() {
        assert_eq!(levenshtein("", "abc"), 3);
    }

    #[test]
    fn levenshtein_empty_both() {
        assert_eq!(levenshtein("", ""), 0);
    }

    #[test]
    fn levenshtein_case_sensitive() {
        assert_eq!(levenshtein("ABC", "abc"), 3);
    }

    #[test]
    fn levenshtein_unicode() {
        assert_eq!(levenshtein("café", "cafe"), 1);
    }
}

/// Prints formatted search results to stdout.
pub fn print_results(results: &[SearchResult]) {
    if results.is_empty() {
        println!("No results found");
        return;
    }

    if results[0].score == 0 {
        println!("Search Results:");
        for (i, res) in results.iter().enumerate() {
            if res.score > 0 {
                break;
            }
            println!("{}.\t{}\t\t{}", i + 1, res.name, res.path);
        }
    } else {
        println!("Couldn't find results. Did you mean?:");
        for (i, res) in results.iter().enumerate() {
            println!("{}.\t{}\t\t{}", i + 1, res.name, res.path);
        }
    }
}
