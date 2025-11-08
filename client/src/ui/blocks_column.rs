use js_sys::Date;
use leptos::prelude::*;
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
pub fn BlocksColumn(blocks: ReadSignal<Vec<Block>>) -> impl IntoView {
    view! {
        <div style="flex:1; background:#0b1020; color:#e5e7eb; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, Liberation Mono, monospace; font-size:12px; padding:8px; overflow:auto;">
            <For
                each=move || blocks.get()
                key=|block| block.number
                children=move |block: Block| {
                    view! {
                        <div style="padding:8px; margin-bottom:8px; background:#1a1f2e; border-radius:4px; border-left:2px solid #3b82f6;">
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
                    }
                }
            />
        </div>
    }
}
