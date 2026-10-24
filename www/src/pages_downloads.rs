use crate::components::{Footer, InstallRow};
use crate::models::GitHubRelease;
use leptos::*;
use leptos_meta::*;

#[component]
pub fn DownloadsPage() -> impl IntoView {
    let latest_release = create_resource(
        || (),
        |_| async move {
            reqwest::get("https://api.github.com/repos/alexlm78/Kree/releases/latest")
                .await
                .ok()?
                .json::<GitHubRelease>()
                .await
                .ok()
        },
    );

    view! {
        <Title text="Kree | Downloads & Setup" />
        <div class="wrap page-head rise">
            <h1>"Download Kree."</h1>
            <p>"Install from crates.io or Homebrew, or grab a prebuilt binary for your system."</p>
        </div>

        <section class="wrap section" aria-labelledby="pkg" style="padding-top:clamp(56px,8vw,112px)">
            <h2 id="pkg" style="font-size:clamp(30px,4vw,56px)">"Package managers"</h2>
            <div class="dl-list">
                <div class="install"><InstallRow cmd="cargo install kree"/><InstallRow cmd="brew install alexlm78/tap/kree"/></div>
            </div>
        </section>

        <section class="wrap section" aria-labelledby="bin" style="padding-top:clamp(56px,8vw,112px)">
            <h2 id="bin" style="font-size:clamp(30px,4vw,56px)">"Prebuilt binaries"</h2>
            <Transition fallback=|| view! { <p class="fine">"Fetching the latest release…"</p> }>
                {move || latest_release.get().map(|rel| match rel {
                    Some(r) => view! {
                        <p class="fine">"Latest release: " <b>{r.tag_name.clone()}</b></p>
                        <div class="dl-list">
                            {r.assets.into_iter().map(|a| view! {
                                <a class="dl" href=a.browser_download_url>
                                    <b>{a.name}</b>
                                    <span>"download ↓"</span>
                                </a>
                            }).collect_view()}
                        </div>
                    }.into_view(),
                    None => view! {
                        <div class="note">
                            <h3>"Could not load the release list."</h3>
                            <p>"Open the " <a href="https://github.com/alexlm78/Kree/releases/latest" target="_blank" rel="noopener">"latest release on GitHub"</a> " instead."</p>
                        </div>
                    }.into_view(),
                })}
            </Transition>
        </section>

        <section class="wrap section" aria-labelledby="cfg" style="padding-top:clamp(56px,8vw,112px)">
            <h2 id="cfg" style="font-size:clamp(30px,4vw,56px)">"Make it yours"</h2>
            <p class="fine">"Defaults, colors and global ignores live in " <code class="cmd">"~/.kreerc"</code> "."</p>
            <pre class="out" style="margin-top:24px;max-width:520px"><span class="pre">"[colors]"</span>"
dir = "<span class="x">"\"blue\""</span>"
exe = "<span class="x">"\"green\""</span></pre>
        </section>
        <Footer/>
    }
}
