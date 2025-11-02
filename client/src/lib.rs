use crate::api::client::Api;
use futures_util::{pin_mut, StreamExt};
use leptos::task::spawn_local;
use leptos::{leptos_dom::logging::console_error, prelude::*};
use shared::types::chain_config::{ChainConfig, ChainStatus};
use std::rc::Rc;

mod api;

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
            match Api::instance().list_chains().await {
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
        move |_id: u64| refresh()
    };

    let on_action = move |id: u64, action: &'static str| {
        set_error_msg.set(None);
        spawn_local(async move {
            if let Err(e) = Api::instance().post_action(&id, action).await {
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
                    let id = c.id;
                    let cb: Rc<dyn Fn(&'static str)> = Rc::new(move |action| on_action(id, action));
                    view!{ <ChainColumn chain=c on_action=cb.clone() /> }
                } />
            </div>

            {move || {
                show_modal.get().then(|| {
                    let existing = chains.get().clone();
                    let on_close = {
                        let set_show_modal = set_show_modal.clone();
                        Rc::new(move || set_show_modal.set(false))
                    };
                    let on_created: Rc<dyn Fn(u64)> = Rc::new(move |id| on_created(id));
                    let config = modal_config.get();
                    view!{ <NewChainModal config=config existing_chains=existing on_close=on_close on_created=on_created /> }
                })
            }}
        </main>
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    Api::init("".to_string());
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| view! { <App/> });
}

// --- UI Components ---

#[component]
fn TopBar(
    set_show_modal: WriteSignal<bool>,
    set_modal_config: WriteSignal<Option<ChainConfig>>,
) -> impl IntoView {
    view! {
        <div style="display:flex; align-items:center; justify-content:space-between; padding:12px 16px; border-bottom:1px solid #e5e7eb; position:sticky; top:0; background:#fff; z-index:10;">
            <div style="font-weight:600; font-size:18px;">{"Local Chain"}</div>
            <div style="display:flex; gap:8px;">
                <button on:click=move |_| {
                    set_modal_config.set(Some(ChainConfig {
                        name: "Ethereum".to_string(),
                        id: 1,
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
    config: Option<ChainConfig>,
    existing_chains: Vec<ChainConfig>,
    on_close: Rc<dyn Fn()>,
    on_created: Rc<dyn Fn(u64)>,
) -> impl IntoView {
    let config = match config {
        Some(c) => c,
        None => ChainConfig::next(&existing_chains),
    };
    let (name, set_name) = signal(config.name.clone());
    let (chain_id, set_chain_id) = signal(config.id.to_string());
    let (port, set_port) = signal(config.port.to_string());
    let (block_time, set_block_time) = signal(config.block_time.to_string());
    let (error, set_error) = signal(None);
    let (submitting, set_submitting) = signal(false);

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
        if existing_chains
            .iter()
            .any(|e| e.name.eq_ignore_ascii_case(&n))
        {
            return Err("Name must be unique".to_string());
        }

        let _cid: u64 = chain_id
            .get()
            .parse()
            .map_err(|_| "Invalid Chain ID".to_string())?;
        if existing_chains.iter().any(|e| e.id == _cid) {
            return Err("Chain ID must be unique".to_string());
        }

        let _port: u16 = port.get().parse().map_err(|_| "Invalid Port".to_string())?;
        if existing_chains.iter().any(|e| e.port == _port) {
            return Err("Port must be unique".to_string());
        }

        // block_time must be grater greater than 0
        let _bt: u64 = block_time.get().parse().unwrap_or(1);
        if _bt == 0 {
            return Err("Block time must be greater than 0".to_string());
        }
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
            id: chain_id.get().parse().unwrap_or(31337),
            port: port.get().parse().unwrap_or(8545),
            block_time: block_time.get().parse().unwrap_or(0),
            status: ChainStatus::Stopped,
        };
        let on_created_cb = on_created_submit.clone();
        let on_close_cb = on_close_submit.clone();
        spawn_local(async move {
            match Api::instance().create_chain(&cfg).await {
                Ok(new_cfg) => {
                    on_created_cb.as_ref()(new_cfg.id);
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

    let id = chain.id;

    Effect::new({
        move |_| {
            spawn_local(async move {
                match Api::instance().log_stream(id) {
                    Ok(mut es) => {
                        let mut stdout = es.subscribe("message").unwrap();
                        let stderr = es.subscribe("error").unwrap();

                        pin_mut!(stdout);
                        pin_mut!(stderr);

                        while let Some(Ok((_event_type, msg))) = stdout.next().await {
                            if let Some(msg) = msg.data().as_string() {
                                set_logs.update(|v| v.push(msg));
                            } else {
                                console_error(
                                    format!("Error reading SSE message: {:?}", msg).as_ref(),
                                );
                            }
                        }
                    }
                    Err(e) => {
                        console_error(format!("Error reading SSE message: {:?}", e).as_ref());
                    }
                }
            });
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
                    {format!("Chain ID: {}  •  Port: {}  •  Block Time: {}", chain.id, chain.port, chain.block_time)}
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
