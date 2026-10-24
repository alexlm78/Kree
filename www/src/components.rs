use crate::models::GitHubRepo;
use leptos::*;
use leptos_router::*;

#[component]
pub fn Navbar(repo_info: Resource<(), Option<GitHubRepo>>) -> impl IntoView {
    view! {
        <header class="wrap">
            <nav class="nav" aria-label="Main">
                <A href="/" class="wordmark">
                    <img src="/favicon.svg" alt="" width="32" height="32"/>
                    "kree"
                </A>
                <div class="nav-links">
                    <A href="/downloads">"Downloads"</A>
                    <Transition fallback=|| ()>
                        {move || repo_info.get().map(|repo| repo.map(|r| view! {
                            <a class="stars" href="https://github.com/alexlm78/Kree" target="_blank" rel="noopener">
                                "GitHub · " {r.stargazers_count} " stars"
                            </a>
                        }))}
                    </Transition>
                </div>
            </nav>
        </header>
    }
}

/// A command with a copy button. Uses an inline handler to avoid extra web-sys features.
#[component]
pub fn InstallRow(cmd: &'static str) -> impl IntoView {
    let onclick = format!(
        "navigator.clipboard&&navigator.clipboard.writeText('{cmd}');this.textContent='copied';var b=this;setTimeout(function(){{b.textContent='copy'}},1500)"
    );
    view! {
        <div class="install-row">
            <code>{cmd}</code>
            <button type="button" attr:onclick=onclick aria-label=format!("Copy {cmd}")>"copy"</button>
        </div>
    }
}

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="wrap foot">
            <div class="foot-links">
                <A href="/downloads">"Downloads"</A>
                <a href="https://github.com/alexlm78/Kree" target="_blank" rel="noopener">"GitHub"</a>
                <a href="https://crates.io/crates/kree" target="_blank" rel="noopener">"crates.io"</a>
                <a href="https://github.com/alexlm78/Kree/issues" target="_blank" rel="noopener">"Report an issue"</a>
            </div>
            <div class="foot-name" aria-hidden="true">"kree"</div>
            <small>"Open source, written in Rust. Made by alexlm78."</small>
        </footer>
    }
}
