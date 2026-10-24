use crate::components::Footer;
use leptos::*;
use leptos_router::*;

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <div class="wrap nf rise">
            <h1>"404"</h1>
            <p class="lede">"This path is not in the tree."</p>
            <pre class="out" style="margin-top:28px;max-width:520px"><span class="pre">"$ "</span>"kree -f this-page\n"<span class="m">"no matches found"</span></pre>
            <div class="cta-row"><A href="/" class="btn">"Back to the root"</A></div>
        </div>
        <Footer/>
    }
}
