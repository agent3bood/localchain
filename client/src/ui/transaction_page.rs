use crate::api::client::Api;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_navigate, use_params_map};
use shared::types::transaction::Transaction;

#[component]
pub fn TransactionPage() -> impl IntoView {
    let params = use_params_map();
    let chain_id = move || {
        params
            .get()
            .get("chainid")
            .and_then(|v| v.parse::<u64>().ok())
    };
    let transaction_hash = move || params.get().get("transactionhash");

    let (transaction, set_transaction) = signal::<Option<Transaction>>(None);
    let (loading, set_loading) = signal(false);
    let (error_msg, set_error_msg) = signal::<Option<String>>(None);
    let navigation = use_navigate();
    let navigate_home = navigation.clone();
    let navigate_for_branch = navigation.clone();

    Effect::new(move |_| {
        if let (Some(cid), Some(hash)) = (chain_id(), transaction_hash()) {
            set_loading.set(true);
            set_error_msg.set(None);
            set_transaction.set(None);
            let api = Api::instance();
            let hash_for_fetch = hash.clone();
            spawn_local(async move {
                match api.get_transaction(cid, hash_for_fetch).await {
                    Ok(resp) => {
                        set_transaction.set(Some(resp.transaction));
                        set_error_msg.set(None);
                    }
                    Err(err) => {
                        set_error_msg.set(Some(err));
                        set_transaction.set(None);
                    }
                }
                set_loading.set(false);
            });
        }
    });

    view! {
        <div style="font-family: system-ui, -apple-system, Segoe UI, Roboto, Ubuntu, Cantarell, Noto Sans, Helvetica, Arial, Apple Color Emoji, Segoe UI Emoji; padding:16px;">
            <div style="margin-bottom:16px;">
                <button
                    on:click=move |_| navigate_home("/", Default::default())
                    style="background:#2563eb; color:white; border:none; padding:8px 12px; border-radius:6px; cursor:pointer;"
                >
                    {"← Back"}
                </button>
            </div>
            {move || {
                match (chain_id(), transaction_hash()) {
                    (Some(cid), Some(_)) => {
                        if loading.get() {
                            view! {
                                <div style="background:white; border:1px solid #e5e7eb; border-radius:8px; padding:16px; text-align:center;">
                                    <div style="color:#6b7280;">{"Loading transaction..."}</div>
                                </div>
                            }
                                .into_any()
                        } else if let Some(err) = error_msg.get() {
                            view! {
                                <div style="padding:16px; color:#842029; background:#f8d7da; border:1px solid #f5c2c7; border-radius:6px;">
                                    <strong>{"Error: "}</strong>
                                    {err}
                                </div>
                            }
                                .into_any()
                        } else if let Some(tx) = transaction.get() {
                            let navigate_to_block = navigate_for_branch.clone();
                            view! {
                                <div style="display:flex; flex-direction:column; gap:16px;">
                                    <div style="background:white; border:1px solid #e5e7eb; border-radius:8px; padding:16px;">
                                        <h1 style="font-size:24px; font-weight:600; margin-bottom:16px;">
                                            {"Transaction Details"}
                                        </h1>
                                        <div style="display:grid; grid-template-columns:repeat(auto-fit, minmax(220px, 1fr)); gap:12px;">
                                            <div style="padding:12px; background:#f9fafb; border-radius:6px;">
                                                <div style="color:#6b7280; font-size:12px; margin-bottom:4px;">
                                                    {"Hash"}
                                                </div>
                                                <div style="font-size:14px; font-family:monospace; word-break:break-all;">
                                                    {tx.hash.clone()}
                                                </div>
                                            </div>
                                            <div style="padding:12px; background:#f9fafb; border-radius:6px;">
                                                <div style="color:#6b7280; font-size:12px; margin-bottom:4px;">
                                                    {"From"}
                                                </div>
                                                <div style="font-size:14px; font-family:monospace; word-break:break-all;">
                                                    {tx.from.clone()}
                                                </div>
                                            </div>
                                            <div style="padding:12px; background:#f9fafb; border-radius:6px;">
                                                <div style="color:#6b7280; font-size:12px; margin-bottom:4px;">
                                                    {"Block Number"}
                                                </div>
                                                <div style="font-size:14px; font-family:monospace;">
                                                    {tx.block_number}
                                                </div>
                                            </div>
                                            <div style="padding:12px; background:#f9fafb; border-radius:6px;">
                                                <div style="color:#6b7280; font-size:12px; margin-bottom:4px;">
                                                    {"Index"}
                                                </div>
                                                <div style="font-size:14px; font-family:monospace;">
                                                    {tx.index}
                                                </div>
                                            </div>
                                        </div>
                                        <div style="display:flex; justify-content:flex-end; margin-top:16px;">
                                            <button
                                                on:click=move |_| {
                                                    navigate_to_block(
                                                        format!("/{}/{}", cid, tx.block_number).as_str(),
                                                        Default::default(),
                                                    )
                                                }
                                                style="background:#2563eb; color:white; border:none; padding:8px 12px; border-radius:6px; cursor:pointer;"
                                            >
                                                {"View Block"}
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            }
                                .into_any()
                        } else {
                            view! {
                                <div style="background:white; border:1px solid #e5e7eb; border-radius:8px; padding:16px;">
                                    <div style="color:#6b7280;">
                                        {"No transaction data available"}
                                    </div>
                                </div>
                            }
                                .into_any()
                        }
                    }
                    _ => {
                        view! {
                            <div style="padding:16px; color:#842029; background:#f8d7da; border:1px solid #f5c2c7; border-radius:6px;">
                                {"Invalid chain ID or transaction hash"}
                            </div>
                        }
                            .into_any()
                    }
                }
            }}
        </div>
    }
}

