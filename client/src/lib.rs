use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <main>
            <h1>LocalChain</h1>
            <p>Leptos client is running.</p>
        </main>
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| view! { <App/> });
}
