use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::hooks::use_params_map;

#[component]
pub fn BlockPage() -> impl IntoView {
    let params = use_params_map();
    let chainid = move || params.get().get("chainid");
    let blocknumber = move || params.get().get("blocknumber");
    let chain_id = move || chainid().and_then(|v| v.parse::<u64>().ok());
    let block_num = move || blocknumber().and_then(|v| v.parse::<u64>().ok());
    let navigate = use_navigate();

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
                    (Some(cid), Some(bnum)) => {
                        view! {
                            <div style="background:white; border:1px solid #e5e7eb; border-radius:8px; padding:16px;">
                                <h1 style="font-size:24px; font-weight:600; margin-bottom:16px;">
                                    {"Block Details"}
                                </h1>
                                <div style="display:flex; flex-direction:column; gap:8px;">
                                    <div>
                                        <span style="color:#6b7280; font-weight:600;">
                                            {"Chain ID: "}
                                        </span>
                                        <span>{cid}</span>
                                    </div>
                                    <div>
                                        <span style="color:#6b7280; font-weight:600;">
                                            {"Block Number: "}
                                        </span>
                                        <span>{bnum}</span>
                                    </div>
                                </div>
                            </div>
                        }
                            .into_any()
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
