use leptos::*;

#[component]
pub fn HomePage() -> impl IntoView {
    let all_files = vec![("", "src/main.rs"), ("", "src/tree.rs"), ("⚙", "Cargo.toml"), ("", "README.md")];
    let (search, set_search) = create_signal(String::new());

    let filtered = move || {
        let s = search.get().to_lowercase();
        all_files.iter()
            .filter(|(_, f)| f.to_lowercase().contains(&s))
            .cloned()
            .collect::<Vec<_>>()
    };

    view! {
        <div class="animate-in fade-in duration-700">
            <header class="max-w-5xl mx-auto py-20 px-6 text-center">
                <h2 class="text-6xl font-black text-white mb-6 tracking-tight">"Pure Rust Tree Explorer"</h2>
                <p class="text-[#8b949e] text-xl max-w-2xl mx-auto mb-10">
                    "Fast, recursive directory visualization with built-in fuzzy search and TUI mode."
                </p>
                <code class="bg-black p-4 rounded-lg border border-[#30363d] text-green-400 text-sm">"cargo install kree"</code>
            </header>

            <section class="max-w-2xl mx-auto px-6 pb-24">
                <div class="bg-[#161b22] p-4 rounded-t-xl border-x border-t border-[#30363d]">
                    <input 
                        type="text" placeholder="Fuzzy search..."
                        class="w-full bg-[#0d1117] border border-[#30363d] rounded-lg px-4 py-2 focus:outline-none focus:border-blue-500 text-white"
                        on:input=move |ev| set_search.set(event_target_value(&ev))
                    />
                </div>
                <div class="bg-[#0d1117] border border-[#30363d] rounded-b-xl p-6 min-h-[200px]">
                    <ul class="space-y-3">
                        <For each=filtered key=|(_, f)| f.to_string() children=|(i, f)| view! {
                            <li class="flex items-center gap-3"><span class="text-blue-400 w-6 text-center">{i}</span>{f}</li>
                        }/>
                    </ul>
                </div>
            </section>
        </div>
    }
}
