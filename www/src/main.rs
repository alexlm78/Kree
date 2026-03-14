mod components;
mod models;
mod pages_404;
mod pages_downloads;
mod pages_home;

use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::components::Navbar;
use crate::models::GitHubRepo;
use crate::pages_404::NotFoundPage;
use crate::pages_downloads::DownloadsPage;
use crate::pages_home::HomePage;

#[component]
fn App() -> impl IntoView {
    // 1. PRIMERO: Inicializar el contexto de metadatos
    provide_meta_context();

    let repo_info = create_resource(
        || (),
        |_| async move {
            reqwest::get("https://api.github.com/repos/alexlm78/Kree")
                .await
                .ok()?
                .json::<GitHubRepo>()
                .await
                .ok()
        },
    );

    view! {
        // 2. SEGUNDO: El Router debe envolver todo lo que use rutas o metadatos
        <Router>
            // Configuramos el idioma en el tag <html> del navegador
            <Html lang="en" />

            // Título por defecto si las páginas no definen uno
            <Title text="Kree | Tree Visualizer" />

            <div class="min-h-screen bg-[#0d1117] text-[#c9d1d9] font-mono">
                <Navbar repo_info=repo_info />
                <main>
                    <Routes>
                        <Route path="/" view=HomePage />
                        <Route path="/downloads" view=DownloadsPage />
                        <Route path="/*any" view=NotFoundPage />
                    </Routes>
                </main>
            </div>
        </Router>
    }
}

fn main() {
    // Opcional: para ver errores de Rust en la consola del navegador
    // _ = console_log::init_with_level(log::Level::Debug);
    // console_error_panic_hook::set_once();

    mount_to_body(|| view! { <App /> })
}
