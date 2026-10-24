use crate::components::{Footer, InstallRow};
use leptos::*;
use leptos_router::*;

/// Kree's own repo, depth-first. (depth, name, is_dir)
const ENTRIES: &[(usize, &str, bool)] = &[
    (0, "docs", true),
    (0, "kree_cli", true),
    (1, "src", true),
    (2, "config.rs", false),
    (2, "ignore.rs", false),
    (2, "main.rs", false),
    (2, "render.rs", false),
    (2, "search.rs", false),
    (2, "tree.rs", false),
    (2, "tui.rs", false),
    (1, "Cargo.toml", false),
    (0, "www", true),
    (1, "src", true),
    (2, "components.rs", false),
    (2, "main.rs", false),
    (2, "pages_home.rs", false),
    (1, "index.html", false),
    (0, "Cargo.toml", false),
    (0, "README.md", false),
];

/// Subsequence fuzzy match. Returns matched char indices, or None.
pub fn fuzzy(query: &str, name: &str) -> Option<Vec<usize>> {
    let mut hits = Vec::new();
    let mut chars = name.chars().enumerate();
    for q in query.chars().filter(|c| !c.is_whitespace()) {
        let q = q.to_ascii_lowercase();
        loop {
            let (i, c) = chars.next()?;
            if c.to_ascii_lowercase() == q {
                hits.push(i);
                break;
            }
        }
    }
    Some(hits)
}

struct Row {
    prefix: String,
    name: &'static str,
    is_dir: bool,
    hits: Vec<usize>,
}

fn build_rows(query: &str) -> Vec<Row> {
    let n = ENTRIES.len();
    let mut keep = vec![query.trim().is_empty(); n];
    let mut hits: Vec<Vec<usize>> = vec![Vec::new(); n];
    if !query.trim().is_empty() {
        for (i, (depth, name, is_dir)) in ENTRIES.iter().enumerate() {
            if let Some(h) = fuzzy(query, name) {
                hits[i] = h;
                keep[i] = true;
                // keep every ancestor so the tree stays connected
                let mut want = *depth;
                for j in (0..i).rev() {
                    if want == 0 {
                        break;
                    }
                    if ENTRIES[j].0 < want {
                        keep[j] = true;
                        want = ENTRIES[j].0;
                    }
                }
                let _ = is_dir;
            }
        }
    }
    let shown: Vec<usize> = (0..n).filter(|&i| keep[i]).collect();
    let mut rows = Vec::new();
    let mut last_at: Vec<bool> = Vec::new();
    for (pos, &i) in shown.iter().enumerate() {
        let (depth, name, is_dir) = ENTRIES[i];
        let is_last = !shown[pos + 1..]
            .iter()
            .take_while(|&&j| ENTRIES[j].0 >= depth)
            .any(|&j| ENTRIES[j].0 == depth);
        last_at.truncate(depth);
        let mut prefix = String::new();
        for &l in &last_at {
            prefix.push_str(if l { "    " } else { "│   " });
        }
        prefix.push_str(if is_last { "└── " } else { "├── " });
        last_at.push(is_last);
        rows.push(Row {
            prefix,
            name,
            is_dir,
            hits: std::mem::take(&mut hits[i]),
        });
    }
    rows
}

#[component]
fn TreeRow(row: Row) -> impl IntoView {
    let cls = if row.is_dir {
        "d"
    } else if row.name.ends_with(".rs") {
        "x"
    } else {
        ""
    };
    let name = row
        .name
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if row.hits.contains(&i) {
                view! { <mark>{c.to_string()}</mark> }.into_view()
            } else {
                c.to_string().into_view()
            }
        })
        .collect_view();
    view! { <span class="pre">{row.prefix}</span><span class=cls>{name}</span>"\n" }
}

#[component]
pub fn HomePage() -> impl IntoView {
    let (query, set_query) = create_signal(String::new());
    let rows = move || build_rows(&query.get());
    let keyed = move || rows().into_iter().enumerate().collect::<Vec<_>>();
    let row_key = |(i, r): &(usize, Row)| format!("{i}{}{}", r.prefix, r.name);
    let render_row = |(_, r): (usize, Row)| view! { <TreeRow row=r/> };
    let count = move || rows().iter().filter(|r| !r.is_dir).count();

    view! {
        <div class="hero">
            <span class="elbow elbow-v" aria-hidden="true"></span>
            <span class="elbow elbow-h" aria-hidden="true"></span>
            <div class="wrap hero-grid">
                <div class="rise">
                    <h1>"Your files, as a tree."</h1>
                    <p class="lede">
                        "Kree draws directory trees in your terminal, finds files by fuzzy name, and opens a full-screen TUI. One small Rust binary."
                    </p>
                    <div class="cta-row">
                        <a class="btn" href="#install">"cargo install kree"</a>
                        <A href="/downloads" class="btn btn-quiet">"Get a binary"</A>
                    </div>
                    <p class="fine">"Linux, macOS and Windows. Try the search on the right."</p>
                </div>
                <div class="term rise" data-ui role="group" aria-label="Interactive demo of Kree fuzzy search">
                    <label class="term-bar">
                        <b>"kree -f"</b>
                        <input
                            type="text"
                            placeholder="try: tui  or  srch"
                            autocomplete="off" spellcheck="false"
                            aria-label="Fuzzy search the Kree repository"
                            on:input=move |ev| set_query.set(event_target_value(&ev))
                        />
                    </label>
                    <pre class="term-body"><span class="root">"."</span>"\n"
                        <For each=keyed key=row_key children=render_row/>
                    </pre>
                    <div class="term-foot">{count} " files shown"</div>
                </div>
            </div>
        </div>

        <section class="wrap section" aria-labelledby="what">
            <div class="split">
                <h2 id="what">"More than a tree."</h2>
                <ul class="feat">
                    <li><h3>"Fuzzy search"</h3><p>"Find files and directories by approximate name with " <code>"-f"</code> ", ranked by Levenshtein distance."</p></li>
                    <li><h3>"Content search"</h3><p>"Look inside files with grep-like output using " <code>"-g"</code> "."</p></li>
                    <li><h3>"Respects your ignores"</h3><p>"Reads " <code>".kreeignore"</code> " and " <code>".gitignore"</code> " automatically. Dotfiles stay hidden until you pass " <code>"-a"</code> "."</p></li>
                    <li><h3>"Smart depth"</h3><p>"Small directories expand further on their own. Set " <code>"-d"</code> " when you want control."</p></li>
                    <li><h3>"Metadata and export"</h3><p>"Permissions, owner, size and date with " <code>"-l"</code> ". Export JSON, YAML or Markdown with " <code>"-F"</code> "."</p></li>
                    <li><h3>"Fast on big trees"</h3><p>"Parallel traversal with rayon scans large projects in a blink."</p></li>
                    <li><h3>"Looks good"</h3><p>"Blue directories, green executables, 40+ extension colors and optional Nerd Font icons."</p></li>
                    <li><h3>"Yours to configure"</h3><p>"Defaults, colors and global ignores live in " <code>"~/.kreerc"</code> ". Completions for bash, zsh, fish, powershell and elvish."</p></li>
                </ul>
            </div>
        </section>

        <section class="wrap section" aria-labelledby="modes">
            <h2 id="modes">"Three ways to look."</h2>
            <div class="modes">
                <article class="mode">
                    <div class="mode-text">
                        <h3>"Print the tree"</h3>
                        <p>"Box-drawing branches, colors and a smart default depth. Pipe it, paste it, commit it."</p>
                        <code class="cmd">"kree ."</code>
                    </div>
                    <pre class="out" data-ui aria-label="Example output of kree"><span class="d">"."</span>"
"<span class="pre">"├── "</span><span class="d">"kree_cli"</span>"
"<span class="pre">"│   ├── "</span><span class="d">"src"</span>"
"<span class="pre">"│   └── "</span>"Cargo.toml
"<span class="pre">"├── "</span><span class="d">"www"</span>"
"<span class="pre">"├── "</span>"Cargo.toml
"<span class="pre">"└── "</span>"README.md"</pre>
                </article>
                <article class="mode">
                    <div class="mode-text">
                        <h3>"Find by name"</h3>
                        <p>"Misspell it and Kree still finds it. Only the matching branches are shown."</p>
                        <code class="cmd">"kree -f serch"</code>
                    </div>
                    <pre class="out" data-ui aria-label="Example output of kree -f"><span class="d">"."</span>"
"<span class="pre">"└── "</span><span class="d">"kree_cli"</span>"
"<span class="pre">"    └── "</span><span class="d">"src"</span>"
"<span class="pre">"        └── "</span><span class="x">"search.rs"</span></pre>
                </article>
                <article class="mode">
                    <div class="mode-text">
                        <h3>"Open the TUI"</h3>
                        <p>"Navigate with the keyboard, expand and collapse folders, preview files and search inline."</p>
                        <code class="cmd">"kree -t"</code>
                    </div>
                    <pre class="out" data-ui aria-label="Example of the Kree TUI"><span class="m">"/ tui"</span>"
"<span class="d">"▾ src"</span>"
"<span class="pre">"  "</span><span class="m">"▸ tui.rs"</span>"
"<span class="pre">"  "</span>"  tree.rs
"<span class="pre">"enter expand · / search · q quit"</span></pre>
                </article>
            </div>
        </section>

        <section class="band" id="install" aria-labelledby="get">
            <div class="wrap band-grid">
                <h2 id="get">"Try it in thirty seconds."</h2>
                <div>
                    <div class="install">
                        <InstallRow cmd="cargo install kree"/>
                        <InstallRow cmd="brew install alexlm78/tap/kree"/>
                    </div>
                    <A href="/downloads" class="link">"Prefer a prebuilt binary? See downloads →"</A>
                </div>
            </div>
        </section>
        <Footer/>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_is_subsequence() {
        assert_eq!(fuzzy("tui", "tui.rs"), Some(vec![0, 1, 2]));
        assert_eq!(fuzzy("srs", "search.rs"), Some(vec![0, 3, 8]));
        assert!(fuzzy("zz", "tui.rs").is_none());
    }

    #[test]
    fn filter_keeps_ancestors() {
        let rows = build_rows("tui");
        let names: Vec<_> = rows.iter().map(|r| r.name).collect();
        assert_eq!(names, ["kree_cli", "src", "tui.rs"]);
        assert!(rows[2].prefix.ends_with("└── "));
    }
}
