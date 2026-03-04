use leptos::*;
use leptos_router::*;
use crate::models::GitHubRepo;

#[component]
pub fn Navbar(repo_info: Resource<(), Option<GitHubRepo>>) -> impl IntoView {
    view! {
        <nav class="max-w-5xl mx-auto p-6 flex justify-between items-center border-b border-[#30363d]">
            <A href="/" class="flex items-center gap-2 no-underline group">
                <span class="text-blue-500 font-bold group-hover:translate-x-1 transition-transform">" > "</span>
                <h1 class="text-xl font-bold text-white tracking-tighter">"KREE"</h1>
            </A>
            <div class="flex gap-6 items-center">
                <A href="/downloads" class="text-sm hover:text-blue-400 transition no-underline text-gray-400">"Downloads"</A>
                <Transition fallback=|| view! { <span class="text-xs">"..."</span> }>
                    {move || repo_info.get().map(|repo| repo.map(|r| view! { 
                        <a href="https://github.com/alexlm78/Kree" target="_blank" 
                           class="text-xs bg-[#161b22] px-3 py-1 rounded-full border border-[#30363d] text-white no-underline hover:border-yellow-500 transition">
                            "⭐ " {r.stargazers_count}
                        </a>
                    }))}
                </Transition>
            </div>
        </nav>
    }
}
