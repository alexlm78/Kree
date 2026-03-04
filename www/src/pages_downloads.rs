use leptos::*;
use leptos_meta::*;
use crate::models::GitHubRelease;

#[component]
pub fn DownloadsPage() -> impl IntoView {
    let latest_release = create_resource(|| (), |_| async move {
        reqwest::get("https://api.github.com/repos/alexlm78/Kree/releases/latest")
            .await.ok()?.json::<GitHubRelease>().await.ok()
    });

    view! {
        <Title text="Kree | Downloads & Setup" />
        <div class="max-w-5xl mx-auto py-12 px-6 animate-in slide-in-from-bottom-4 duration-500">
            <div class="grid grid-cols-1 lg:grid-cols-3 gap-12">
                <div class="lg:col-span-2">
                    <h2 class="text-3xl font-bold text-white mb-8">"Binary Releases"</h2>
                    <Transition fallback=|| view! { <p>"Fetching..."</p> }>
                        {move || latest_release.get().map(|rel| rel.map(|r| view! {
                            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                                {r.assets.into_iter().map(|a| view! {
                                    <a href=a.browser_download_url class="p-4 bg-[#161b22] border border-[#30363d] rounded-xl hover:border-blue-500 no-underline transition">
                                        <div class="font-bold text-white mb-1">{a.name}</div>
                                        <span class="text-[10px] text-gray-500 uppercase">"Stable Release"</span>
                                    </a>
                                }).collect_view()}
                            </div>
                        }))}
                    </Transition>
                </div>

                <aside class="lg:col-span-1 space-y-6 text-sm">
                    <div class="p-6 bg-[#161b22] border border-[#30363d] rounded-2xl">
                        <h4 class="text-white font-bold mb-4">"Config (~/.kreerc)"</h4>
                        <pre class="text-[10px] text-blue-300">
"[colors]
dir = \"blue\"
exe = \"green\""
                        </pre>
                    </div>
                </aside>
            </div>
        </div>
    }
}
