use leptos::*;
use leptos_router::*;

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <div class="max-w-5xl mx-auto py-20 px-6 text-center font-mono">
            <div class="inline-block bg-[#161b22] border border-red-500/50 p-8 rounded-2xl shadow-2xl">
                <h2 class="text-6xl font-black text-red-500 mb-4">"404"</h2>
                <p class="text-white text-xl mb-6">"[ERROR]: Path not found in tree"</p>

                <div class="bg-black p-4 rounded border border-[#30363d] text-left mb-8">
                    <p class="text-gray-500">"$ kree --check-route"</p>
                    <p class="text-red-400">"Error: The requested directory does not exist."</p>
                </div>

                <A href="/" class="text-blue-400 hover:underline">
                    "center" " Return to root_dir"
                </A>
            </div>
        </div>
    }
}
