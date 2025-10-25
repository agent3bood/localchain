use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{EventSource, MessageEvent};
use shared::{ChainConfig, ChainStatus};

#[component]
pub fn App() -> impl IntoView {
    let (show_modal, set_show_modal) = create_signal(false);
    let (modal_config, set_modal_config) = create_signal::<Option<ChainConfig>>(None);
    let (chains, set_chains) = create_signal::<Vec<ChainConfig>>(vec![]);
    let (loading, set_loading) = create_signal(false);
    let (error_msg, set_error_msg) = create_signal::<Option<String>>(None);

    let refresh = move || {
        set_loading.set(true);
        set_error_msg.set(None);
        spawn_local(async move {
            match api_list_chains().await {
                Ok(list) => {
                    set_chains.set(list);
                }
                Err(e) => set_error_msg.set(Some(e)),
            }
            set_loading.set(false);
        });
    };

    // run once on mount
    Effect::new(move |_| refresh());

    let on_created = {
        let refresh = refresh.clone();
        move |_id: String| refresh()
    };

    let on_action = move |id: String, action: &'static str| {
        set_error_msg.set(None);
        spawn_local(async move {
            if let Err(e) = api_post_action(&id, action).await {
                set_error_msg.set(Some(e));
            }
            // always refresh to reflect latest server state
            refresh();
        });
    };

    view! {
        <main style="font-family: system-ui, -apple-system, Segoe UI, Roboto, Ubuntu, Cantarell, Noto Sans, Helvetica, Arial, Apple Color Emoji, Segoe UI Emoji;">
            <TopBar set_show_modal=set_show_modal set_modal_config=set_modal_config />
            {move || error_msg.get().map(|e| view!{ <div style="margin:8px; padding:8px; color:#842029; background:#f8d7da; border:1px solid #f5c2c7; border-radius:6px;">{e}</div> })}
            {move || if loading.get() { Some(view!{ <div style="margin:8px;">{"Loading..."}</div> }) } else { None }}
            <div style="display:flex; gap:16px; overflow-x:auto; padding:16px;">
                <For each=move || chains.get() key=|c| c.name.clone() children=move |c: ChainConfig| {
                    let id = c.name.clone();
                    let cb: Rc<dyn Fn(&'static str)> = Rc::new(move |action| on_action(id.clone(), action));
                    view!{ <ChainColumn chain=c on_action=cb.clone() /> }
                } />
            </div>

            {move || {
                show_modal.get().then(|| {
                    let existing = chains.get().into_iter().map(|c| c.name.clone()).collect::<Vec<_>>();
                    let on_close = {
                        let set_show_modal = set_show_modal.clone();
                        Rc::new(move || set_show_modal.set(false))
                    };
                    let on_created: Rc<dyn Fn(String)> = Rc::new(move |id| on_created(id));
                    let config = modal_config.get();
                    view!{ <NewChainModal existing_names=existing on_close=on_close on_created=on_created /> }
                })
            }}
        </main>
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| view! { <App/> });
}

// --- API client helpers (scaffold) ---
pub async fn api_list_chains() -> Result<Vec<ChainConfig>, String> {
    let resp = Request::get("/api/chains")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.json::<Vec<ChainConfig>>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn api_create_chain(config: &ChainConfig) -> Result<ChainConfig, String> {
    let resp = Request::post("/api/chains")
        .json(config)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.json::<ChainConfig>().await.map_err(|e| e.to_string())
}

pub async fn api_post_action(chain_id: &str, action: &str) -> Result<(), String> {
    let url = format!("/api/chains/{}/{}", chain_id, action);
    let resp = Request::post(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    Ok(())
}

// --- SSE logs helper (scaffold) ---
pub fn open_log_stream(chain_id: &str) -> Result<EventSource, String> {
    let url = format!("/api/chains/{}/logstream", chain_id);
    EventSource::new(&url).map_err(|e| format!("{e:?}"))
}

pub fn attach_log_handlers(
    es: &EventSource,
    on_message: impl Fn(String) + 'static,
    on_error: impl Fn(String) + 'static,
) {
    let on_message_cb = Closure::wrap(Box::new(move |e: web_sys::Event| {
        if let Some(me) = e.dyn_ref::<MessageEvent>() {
            if let Ok(txt) = me.data().dyn_into::<js_sys::JsString>() {
                on_message(txt.as_string().unwrap_or_default());
            }
        }
    }) as Box<dyn FnMut(_)>);
    es.set_onmessage(Some(on_message_cb.as_ref().unchecked_ref()));
    on_message_cb.forget();

    let on_error_cb = Closure::wrap(Box::new(move |_e: web_sys::Event| {
        on_error("sse_error".to_string());
    }) as Box<dyn FnMut(_)>);
    es.set_onerror(Some(on_error_cb.as_ref().unchecked_ref()));
    on_error_cb.forget();
}

// --- UI Components ---

#[component]
fn TopBar(set_show_modal: WriteSignal<bool>, set_modal_config: WriteSignal<Option<ChainConfig>>) -> impl IntoView {
    view! {
        <div style="display:flex; align-items:center; justify-content:space-between; padding:12px 16px; border-bottom:1px solid #e5e7eb; position:sticky; top:0; background:#fff; z-index:10;">
            <div style="font-weight:600; font-size:18px;">{"Local Chain"}</div>
            <div style="display:flex; gap:8px;">
                <button on:click=move |_| {
                    set_modal_config.set(Some(ChainConfig {
                        name: "Ethereum".to_string(),
                        chain_id: 1,
                        port: 8545,
                        block_time: 1,
                        status: ChainStatus::Stopped,
                    }));
                    set_show_modal.set(true);
                } style="background:none; border:none; padding:8px; border-radius:6px; cursor:pointer;">
                    <img src="/assets/ethereum_logo.svg" alt="New Ethereum Chain" style="width:32px; height:32px;" />
                </button>
                <button on:click=move |_| {
                    set_modal_config.set(None);
                    set_show_modal.set(true);
                } style="background:#2563eb; color:white; border:none; padding:8px 12px; border-radius:6px; cursor:pointer;">{"New Chain"}</button>
            </div>
        </div>
    }
}

#[component]
fn NewChainModal(
    existing_names: Vec<String>,
    on_close: Rc<dyn Fn()>,
    on_created: Rc<dyn Fn(String)>,
) -> impl IntoView {
    let (name, set_name) = create_signal(String::new());
    let (chain_id, set_chain_id) = create_signal(String::from("31337"));
    let (port, set_port) = create_signal(String::from("8545"));
    let (block_time, set_block_time) = create_signal(String::from("1"));
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (submitting, set_submitting) = create_signal(false);

    // clones for handlers to avoid moving the originals
    let on_close_submit = on_close.clone();
    let on_created_submit = on_created.clone();
    let on_close_cancel = on_close.clone();

    let validate = move || {
        let n = name.get();
        if n.trim().is_empty() {
            return Err("Name is required".to_string());
        }
        if !n
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err("Name must be alphanumeric (dash/underscore allowed)".to_string());
        }
        if existing_names.iter().any(|e| e.eq_ignore_ascii_case(&n)) {
            return Err("Name must be unique".to_string());
        }
        let _cid: u64 = chain_id
            .get()
            .parse()
            .map_err(|_| "Invalid Chain ID".to_string())?;
        let _port: u16 = port.get().parse().map_err(|_| "Invalid Port".to_string())?;
        // block_time is optional; treat empty or invalid as 0 during submission
        let _bt: u64 = block_time.get().parse().unwrap_or(0);
        Ok(())
    };

    let submit = move |_| {
        set_error.set(None);
        if let Err(e) = validate() {
            set_error.set(Some(e));
            return;
        }
        set_submitting.set(true);
        let cfg = ChainConfig {
            name: name.get(),
            chain_id: chain_id.get().parse().unwrap_or(31337),
            port: port.get().parse().unwrap_or(8545),
            block_time: block_time.get().parse().unwrap_or(0),
            status: ChainStatus::Stopped,
        };
        let on_created_cb = on_created_submit.clone();
        let on_close_cb = on_close_submit.clone();
        spawn_local(async move {
            match api_create_chain(&cfg).await {
                Ok(new_cfg) => {
                    on_created_cb.as_ref()(new_cfg.name.clone());
                    on_close_cb.as_ref()();
                }
                Err(e) => set_error.set(Some(e)),
            }
            set_submitting.set(false);
        });
    };

    view! {
        <div style="position:fixed; inset:0; background:rgba(0,0,0,0.4); display:flex; align-items:center; justify-content:center;">
            <div style="background:white; padding:16px; width:420px; border-radius:8px; box-shadow:0 10px 25px rgba(0,0,0,0.2);">
                <div style="font-weight:600; font-size:16px; margin-bottom:12px;">New Chain</div>
                {move || error.get().map(|e| view!{ <div style="margin-bottom:8px; padding:8px; color:#842029; background:#f8d7da; border:1px solid #f5c2c7; border-radius:6px;">{e}</div> })}
                <div style="display:flex; flex-direction:column; gap:8px;">
                    <label>Name<input prop:value=move || name.get() on:input=move |ev| set_name.set(event_target_value(&ev)) style="width:100%; padding:6px; border:1px solid #e5e7eb; border-radius:6px;" /></label>
                    <label>Chain ID<input prop:value=move || chain_id.get() on:input=move |ev| set_chain_id.set(event_target_value(&ev)) inputmode="numeric" style="width:100%; padding:6px; border:1px solid #e5e7eb; border-radius:6px;" /></label>
                    <label>Port<input prop:value=move || port.get() on:input=move |ev| set_port.set(event_target_value(&ev)) inputmode="numeric" style="width:100%; padding:6px; border:1px solid #e5e7eb; border-radius:6px;" /></label>
                    <label>Block Time (s)<input prop:value=move || block_time.get() on:input=move |ev| set_block_time.set(event_target_value(&ev)) inputmode="numeric" style="width:100%; padding:6px; border:1px solid #e5e7eb; border-radius:6px;" /></label>
                </div>
                <div style="display:flex; gap:8px; justify-content:flex-end; margin-top:12px;">
                    {
                        let on_close_cancel = on_close_cancel.clone();
                        view!{ <button on:click=move |_| on_close_cancel.as_ref()() style="background:white; border:1px solid #d1d5db; padding:8px 12px; border-radius:6px; cursor:pointer;">{"Cancel"}</button> }
                    }
                    <button disabled=move || submitting.get() on:click=submit style="background:#2563eb; color:white; border:none; padding:8px 12px; border-radius:6px; cursor:pointer;">{move || if submitting.get() { "Starting..." } else { "Start" }}</button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ChainColumn(chain: ChainConfig, on_action: Rc<dyn Fn(&'static str)>) -> impl IntoView {
    let (show_info, set_show_info) = create_signal(false);
    let (logs, set_logs) = create_signal(Vec::<String>::new());
    let es: Rc<RefCell<Option<EventSource>>> = Rc::new(RefCell::new(None));

    let id = chain.name.clone();
    let status = chain.status;

    Effect::new({
        let es = Rc::clone(&es);
        move |_| {
            let current_status = status;
            if current_status == ChainStatus::Running {
                if es.borrow().is_none() {
                    if let Ok(src) = open_log_stream(&id) {
                        let setter = set_logs.clone();
                        attach_log_handlers(
                            &src,
                            move |line| {
                                setter.update(|v| v.push(line));
                            },
                            move |_err| {
                                // ignore; browser will attempt auto-reconnect
                            },
                        );
                        *es.borrow_mut() = Some(src);
                    }
                }
            } else {
                if let Some(src) = es.borrow_mut().take() {
                    src.close();
                }
            }
        }
    });

    let status_text = match chain.status {
        ChainStatus::Stopped => "🔴 Stopped",
        ChainStatus::Running => "🟢 Running",
        ChainStatus::Starting => "🟡 Starting",
        ChainStatus::Error => "🟠 Error",
    };

    let can_start = matches!(chain.status, ChainStatus::Stopped);
    let can_stop = matches!(chain.status, ChainStatus::Running);
    let can_restart = matches!(chain.status, ChainStatus::Running | ChainStatus::Error);

    view! {
        <div style="min-width:380px; border:1px solid #e5e7eb; border-radius:8px; overflow:hidden; display:flex; flex-direction:column;">
            <div style="display:flex; align-items:center; justify-content:space-between; padding:8px 10px; background:#f9fafb; border-bottom:1px solid #e5e7eb;">
                <div style="font-weight:600;">{chain.name.clone()}</div>
                <div style="display:flex; align-items:center; gap:8px;">
                    <span style="font-size:12px; padding:2px 6px; border:1px solid #e5e7eb; border-radius:9999px; background:white;">{status_text}</span>
                    { let on_action = on_action.clone(); view!{ <button disabled=move || !can_start on:click=move |_| on_action("start") style="padding:6px 8px; border:1px solid #d1d5db; background:white; border-radius:6px; cursor:pointer;">{"Start"}</button> } }
                    { let on_action = on_action.clone(); view!{ <button disabled=move || !can_stop on:click=move |_| on_action("stop") style="padding:6px 8px; border:1px solid #d1d5db; background:white; border-radius:6px; cursor:pointer;">{"Stop"}</button> } }
                    { let on_action = on_action.clone(); view!{ <button disabled=move || !can_restart on:click=move |_| on_action("restart") style="padding:6px 8px; border:1px solid #d1d5db; background:white; border-radius:6px; cursor:pointer;">{"Restart"}</button> } }
                    { let on_action = on_action.clone(); view! { <button on:click=move |_| on_action("delete") style="padding:6px 8px; border:1px solid #d1d5db; background:white; border-radius:6px; cursor:pointer;">{"Delete"}</button> } }
                    <button on:click=move |_| set_show_info.update(|v| *v = !*v) style="padding:6px 8px; border:1px solid #d1d5db; background:white; border-radius:6px; cursor:pointer;">{"Info"}</button>
                    <button on:click=move |_| set_logs.set(vec![]) style="padding:6px 8px; border:1px solid #d1d5db; background:white; border-radius:6px; cursor:pointer;">{"Clear Log"}</button>
                </div>
            </div>
            {move || show_info.get().then(|| {
                view!{ <div style="padding:8px 10px; border-bottom:1px solid #e5e7eb; font-size:12px; color:#374151;">
                    {format!("Chain ID: {}  •  Port: {}  •  Block Time: {}", chain.chain_id, chain.port, chain.block_time)}
                </div> }
            })}
            <div style="flex:1; background:#0b1020; color:#e5e7eb; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, Liberation Mono, monospace; font-size:12px; padding:8px; white-space:pre-wrap; overflow:auto;">
                <For each=move || logs.get() key=|line| line.clone() children=move |line: String| {
                    view!{ <div>{line}</div> }
                } />
            </div>
        </div>
    }
}
