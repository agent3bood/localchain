use leptos::prelude::*;

#[component]
pub fn LogsColumn(logs: ReadSignal<Vec<String>>) -> impl IntoView {
    view! {
        <div style="flex:1; background:#0b1020; color:#e5e7eb; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, Liberation Mono, monospace; font-size:12px; padding:8px; overflow:auto;">
            <For
                each=move || logs.get()
                key=|log| log.clone()
                children=move |log: String| {
                    view! { <div>{log}</div> }
                }
            />
        </div>
    }
}
