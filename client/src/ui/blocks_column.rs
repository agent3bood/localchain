use js_sys::Date;
use leptos::prelude::*;
use leptos_router::components::A;
use shared::types::block::Block;

fn format_timestamp(timestamp: u64) -> String {
    let date = Date::new(&wasm_bindgen::JsValue::from_f64((timestamp * 1000) as f64));
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        date.get_full_year(),
        date.get_month() + 1,
        date.get_date(),
        date.get_hours(),
        date.get_minutes(),
        date.get_seconds()
    )
}

fn truncate_hash(hash: &str, len: usize) -> String {
    if hash.len() <= len {
        hash.to_string()
    } else {
        format!("{}...{}", &hash[..len / 2], &hash[hash.len() - len / 2..])
    }
}

#[component]
pub fn BlocksColumn(blocks: ReadSignal<Vec<Block>>, chainid: u64) -> impl IntoView {
    view! {
        <div style="flex:1; background:#0b1020; color:#e5e7eb; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, Liberation Mono, monospace; font-size:12px; padding:8px; overflow:auto;">
            <For
                each=move || blocks.get()
                key=|block| block.number
                children=move |block: Block| {
                    let chainid = chainid;
                    let block_number = block.number;
                    let (is_hovered, set_is_hovered) = signal(false);
                    view! {
                        <div style="text-decoration:none; color:inherit; display:block;">
                            <A href=format!("/{}/{}", chainid, block_number)>
                                <div
                                    style=move || {
                                        format!(
                                            "padding:8px; margin-bottom:8px; background:{}; border-radius:4px; border-left:2px solid #3b82f6; cursor:pointer; transition:background 0.2s;",
                                            if is_hovered.get() { "#252a3a" } else { "#1a1f2e" },
                                        )
                                    }
                                    on:mouseenter=move |_| set_is_hovered.set(true)
                                    on:mouseleave=move |_| set_is_hovered.set(false)
                                >
                                    <div style="display:flex; flex-direction:column; gap:4px;">
                                        <div style="display:flex; align-items:center; gap:8px;">
                                            <span style="color:#9ca3af; font-weight:600;">Block:</span>
                                            <span style="color:#60a5fa;">{block.number}</span>
                                        </div>
                                        <div style="display:flex; align-items:center; gap:8px;">
                                            <span style="color:#9ca3af; font-weight:600;">Hash:</span>
                                            <span style="color:#e5e7eb; font-family:monospace; font-size:11px;">
                                                {truncate_hash(&block.hash, 16)}
                                            </span>
                                        </div>
                                        <div style="display:flex; align-items:center; gap:8px;">
                                            <span style="color:#9ca3af; font-weight:600;">Time:</span>
                                            <span style="color:#e5e7eb;">
                                                {format_timestamp(block.time)}
                                            </span>
                                        </div>
                                        <div style="display:flex; align-items:center; gap:8px;">
                                            <span style="color:#9ca3af; font-weight:600;">
                                                Transactions:
                                            </span>
                                            <span style="color:#e5e7eb;">{block.transactions}</span>
                                        </div>
                                    </div>
                                </div>
                            </A>
                        </div>
                    }
                }
            />
        </div>
    }
}
