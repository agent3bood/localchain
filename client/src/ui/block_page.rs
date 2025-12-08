use crate::api::client::Api;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;
use leptos_router::hooks::use_params_map;
use shared::types::block_response::BlockResponse;
use shared::types::transaction::Transaction;

#[component]
pub fn BlockPage() -> impl IntoView {
    let params = use_params_map();
    let chainid = move || params.get().get("chainid");
    let blocknumber = move || params.get().get("blocknumber");
    let chain_id = move || chainid().and_then(|v| v.parse::<u64>().ok());
    let block_num = move || blocknumber().and_then(|v| v.parse::<u64>().ok());
    let navigate = use_navigate();

    let (block_data, set_block_data) = signal::<Option<BlockResponse>>(None);
    let (loading, set_loading) = signal(false);
    let (error_msg, set_error_msg) = signal::<Option<String>>(None);

    Effect::new(move |_| {
        if let (Some(cid), Some(bnum)) = (chain_id(), block_num()) {
            set_loading.set(true);
            set_error_msg.set(None);
            set_block_data.set(None);
            let api = Api::instance();
            spawn_local(async move {
                match api.get_block(cid, bnum).await {
                    Ok(data) => {
                        set_block_data.set(Some(data));
                        set_error_msg.set(None);
                    }
                    Err(e) => {
                        set_error_msg.set(Some(e));
                        set_block_data.set(None);
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
                    on:click=move |_| navigate("/", Default::default())
                    style="background:#2563eb; color:white; border:none; padding:8px 12px; border-radius:6px; cursor:pointer;"
                >
                    {"← Back"}
                </button>
            </div>
            {move || {
                match (chain_id(), block_num()) {
                    (Some(cid), Some(_)) => {
                        if loading.get() {
                            view! {
                                <div style="background:white; border:1px solid #e5e7eb; border-radius:8px; padding:16px; text-align:center;">
                                    <div style="color:#6b7280;">{"Loading block data..."}</div>
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
                        } else if let Some(data) = block_data.get() {
                            let block = data.block.clone();
                            let transactions = data.transactions.clone();
                            view! {
                                <div style="display:flex; flex-direction:column; gap:16px;">
                                    <div style="background:white; border:1px solid #e5e7eb; border-radius:8px; padding:16px;">
                                        <h1 style="font-size:24px; font-weight:600; margin-bottom:16px;">
                                            {"Block Details"}
                                        </h1>
                                        <div style="display:grid; grid-template-columns:repeat(auto-fit, minmax(250px, 1fr)); gap:12px;">
                                            <div style="padding:8px; background:#f9fafb; border-radius:4px;">
                                                <div style="color:#6b7280; font-size:12px; font-weight:600; margin-bottom:4px;">
                                                    {"Chain ID"}
                                                </div>
                                                <div style="font-size:14px; font-family:monospace;">
                                                    {cid}
                                                </div>
                                            </div>
                                            <div style="padding:8px; background:#f9fafb; border-radius:4px;">
                                                <div style="color:#6b7280; font-size:12px; font-weight:600; margin-bottom:4px;">
                                                    {"Block Number"}
                                                </div>
                                                <div style="font-size:14px; font-family:monospace;">
                                                    {block.number}
                                                </div>
                                            </div>
                                            <div style="padding:8px; background:#f9fafb; border-radius:4px;">
                                                <div style="color:#6b7280; font-size:12px; font-weight:600; margin-bottom:4px;">
                                                    {"Hash"}
                                                </div>
                                                <div style="font-size:12px; font-family:monospace; word-break:break-all;">
                                                    {block.hash.clone()}
                                                </div>
                                            </div>
                                            <div style="padding:8px; background:#f9fafb; border-radius:4px;">
                                                <div style="color:#6b7280; font-size:12px; font-weight:600; margin-bottom:4px;">
                                                    {"Beneficiary"}
                                                </div>
                                                <div style="font-size:12px; font-family:monospace; word-break:break-all;">
                                                    {block.beneficiary.clone()}
                                                </div>
                                            </div>
                                            <div style="padding:8px; background:#f9fafb; border-radius:4px;">
                                                <div style="color:#6b7280; font-size:12px; font-weight:600; margin-bottom:4px;">
                                                    {"Gas Limit"}
                                                </div>
                                                <div style="font-size:14px; font-family:monospace;">
                                                    {block.gas_limit}
                                                </div>
                                            </div>
                                            <div style="padding:8px; background:#f9fafb; border-radius:4px;">
                                                <div style="color:#6b7280; font-size:12px; font-weight:600; margin-bottom:4px;">
                                                    {"Gas Used"}
                                                </div>
                                                <div style="font-size:14px; font-family:monospace;">
                                                    {block.gas_used}
                                                </div>
                                            </div>
                                            <div style="padding:8px; background:#f9fafb; border-radius:4px;">
                                                <div style="color:#6b7280; font-size:12px; font-weight:600; margin-bottom:4px;">
                                                    {"Timestamp"}
                                                </div>
                                                <div style="font-size:14px; font-family:monospace;">
                                                    {block.time}
                                                </div>
                                            </div>
                                            <div style="padding:8px; background:#f9fafb; border-radius:4px;">
                                                <div style="color:#6b7280; font-size:12px; font-weight:600; margin-bottom:4px;">
                                                    {"Nonce"}
                                                </div>
                                                <div style="font-size:12px; font-family:monospace; word-break:break-all;">
                                                    {block.nonce.clone()}
                                                </div>
                                            </div>
                                            <div style="padding:8px; background:#f9fafb; border-radius:4px;">
                                                <div style="color:#6b7280; font-size:12px; font-weight:600; margin-bottom:4px;">
                                                    {"Transaction Count"}
                                                </div>
                                                <div style="font-size:14px; font-family:monospace;">
                                                    {block.transactions}
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                    <div style="background:white; border:1px solid #e5e7eb; border-radius:8px; padding:16px;">
                                        <h2 style="font-size:20px; font-weight:600; margin-bottom:16px;">
                                            {"Transactions ("}{transactions.len()}{")"}
                                        </h2>
                                        {if transactions.is_empty() {
                                            view! {
                                                <div style="padding:16px; text-align:center; color:#6b7280;">
                                                    {"No transactions in this block"}
                                                </div>
                                            }
                                                .into_any()
                                        } else {
                                            let tx_list = transactions.clone();
                                            view! {
                                                <div style="display:flex; flex-direction:column; gap:8px;">
                                                    <For
                                                        each=move || {
                                                            tx_list.clone().into_iter().enumerate().collect::<Vec<_>>()
                                                        }
                                                        key=|(idx, _)| *idx
                                                        children=move |(idx, tx): (usize, Transaction)| {
                                                            view! { <TransactionDetails tx=tx idx=idx chain_id=cid /> }
                                                                .into_any()
                                                        }
                                                    />
                                                </div>
                                            }
                                                .into_any()
                                        }}
                                    </div>
                                </div>
                            }
                                .into_any()
                        } else {
                            view! {
                                <div style="background:white; border:1px solid #e5e7eb; border-radius:8px; padding:16px;">
                                    <div style="color:#6b7280;">{"No block data available"}</div>
                                </div>
                            }
                                .into_any()
                        }
                    }
                    _ => {
                        view! {
                            <div style="padding:16px; color:#842029; background:#f8d7da; border:1px solid #f5c2c7; border-radius:6px;">
                                {"Invalid chain ID or block number"}
                            </div>
                        }
                            .into_any()
                    }
                }
            }}
        </div>
    }
}

#[component]
pub fn TransactionDetails(tx: Transaction, idx: usize, chain_id: u64) -> impl IntoView {
    let hash = tx.hash.clone();
    let from = tx.from.clone();
    let block_number = tx.block_number;
    let link = format!("/{}/transactions/{}", chain_id, hash);
    view! {
        <A href=link>
            <div style="text-decoration:none; color:inherit; display:flex; padding:12px; background:#f9fafb; border:1px solid #e5e7eb; border-radius:6px; align-items:center; gap:12px;">
                <div style="color:#6b7280; font-weight:600; min-width:80px;">
                    {format!("#{}", idx + 1)}
                </div>
                <div style="flex:1;">
                    <div style="color:#6b7280; font-size:12px; margin-bottom:4px;">{"Hash"}</div>
                    <div style="font-size:12px; font-family:monospace; word-break:break-all;">
                        {hash}
                    </div>
                    <div style="color:#6b7280; font-size:12px; margin-top:8px;">{"From"}</div>
                    <div style="font-size:12px; font-family:monospace; word-break:break-all;">
                        {from}
                    </div>
                </div>
                <div style="text-align:right; min-width:110px;">
                    <div style="color:#6b7280; font-size:12px;">{"Block"}</div>
                    <div style="font-size:14px; font-family:monospace;">{block_number}</div>
                </div>
            </div>
        </A>
    }
}
