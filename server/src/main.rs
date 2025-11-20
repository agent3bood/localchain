use crate::anvil::process::AnvilProcess;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{sse, Html, IntoResponse, Sse},
    routing::{get, post},
    Json, Router,
};
use futures::Stream;
use shared::types::{
    block::Block,
    block_response::BlockResponse,
    chain_config::{ChainConfig, ChainStatus},
    transaction::Transaction,
};
use std::convert::Infallible;
use std::pin::Pin;
use std::{collections::HashMap, net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::sync::{broadcast, Mutex};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tower_http::services::ServeDir;

mod anvil;

#[derive(Clone)]
struct AppState {
    client_dist: PathBuf,
    manager: Arc<ChainsManager>,
}

struct ChainEntry {
    id: u64,
    config: ChainConfig,
    log_tx: Arc<broadcast::Sender<String>>,
    block_tx: Arc<broadcast::Sender<Block>>,
    process: Arc<Mutex<AnvilProcess>>,
}

#[derive(Default)]
struct ChainsManager {
    /// id: ChainEntry
    inner: Mutex<HashMap<u64, ChainEntry>>,
}

impl ChainsManager {
    async fn list(&self) -> Vec<ChainConfig> {
        let map = self.inner.lock().await;
        map.values().map(|c| c.config.clone()).collect()
    }

    async fn create(&self, cfg: ChainConfig) -> Result<u64, String> {
        let mut map = self.inner.lock().await;
        if map.contains_key(&cfg.id) {
            return Err("name already exists".into());
        }
        let (log_tx, _log_rx) = broadcast::channel(1024);
        let (block_tx, _block_rx) = broadcast::channel(1024);
        let log_tx = Arc::new(log_tx);
        let block_tx = Arc::new(block_tx);
        let process = AnvilProcess::new(
            cfg.name.clone(),
            cfg.id,
            cfg.port,
            cfg.block_time,
            log_tx.clone(),
            block_tx.clone(),
            cfg.fork_url.clone(),
        );
        let entry = ChainEntry {
            id: cfg.id,
            config: cfg,
            log_tx: log_tx,
            block_tx: block_tx,
            process: Arc::new(Mutex::new(process)),
        };
        let id = entry.id.clone();
        map.insert(id, entry);
        drop(map);
        Ok(id)
    }

    async fn start(&self, id: &u64) -> Result<(), String> {
        let mut map = self.inner.lock().await;
        let Some(entry) = map.get_mut(id) else {
            return Err("not found".into());
        };
        entry.config.status = ChainStatus::Starting;
        let mut process = entry.process.lock().await;
        match process.start().await {
            Ok(()) => {
                entry.config.status = ChainStatus::Running;
                Ok(())
            }
            Err(e) => {
                entry.config.status = ChainStatus::Error;
                Err(e)
            }
        }
    }

    async fn stop(&self, id: &u64) -> Result<(), String> {
        let mut map = self.inner.lock().await;
        let Some(entry) = map.get_mut(id) else {
            return Err("not found".into());
        };
        let mut process = entry.process.lock().await;
        match process.stop().await {
            Ok(()) => {
                entry.config.status = ChainStatus::Stopped;
                let _ = entry.log_tx.send("[manager] stopped".into());
                Ok(())
            }
            Err(e) => {
                entry.config.status = ChainStatus::Error;
                Err(e)
            }
        }
    }

    async fn restart(&self, id: &u64) -> Result<(), String> {
        self.stop(id).await?;
        self.start(id).await?;
        Ok(())
    }

    async fn delete(&self, id: &u64) -> Result<(), String> {
        let process = {
            let mut map = self.inner.lock().await;
            let Some(entry) = map.get_mut(id) else {
                return Err("not found".into());
            };
            entry.process.clone()
        };
        process.lock().await.stop().await?;

        let mut map = self.inner.lock().await;
        map.remove(id);
        Ok(())
    }

    async fn subscribe_logs(&self, id: &u64) -> Result<broadcast::Receiver<String>, String> {
        let map = self.inner.lock().await;
        let Some(entry) = map.get(id) else {
            return Err("not found".into());
        };
        Ok(entry.log_tx.subscribe())
    }

    async fn subscribe_blocks(&self, id: &u64) -> Result<broadcast::Receiver<Block>, String> {
        let map = self.inner.lock().await;
        let Some(entry) = map.get(id) else {
            return Err("not found".into());
        };
        Ok(entry.block_tx.subscribe())
    }

    async fn get_block(
        &self,
        chain_id: &u64,
        block_number: u64,
    ) -> Result<(Block, Vec<Transaction>), String> {
        let process = {
            let map = self.inner.lock().await;
            let Some(entry) = map.get(chain_id) else {
                return Err("chain not found".into());
            };
            entry.process.clone()
        };
        let process = process.lock().await;
        process.get_block_with_transactions(block_number).await
    }
}

#[tokio::main]
async fn main() {
    let client_dist = std::env::var("CLIENT_DIST")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            // default to ../client/dist for trunk builds
            let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            p.pop();
            p.push("client");
            p.push("dist");
            p
        });

    let state = AppState {
        client_dist: client_dist.clone(),
        manager: Arc::new(ChainsManager::default()),
    };

    // Serve static assets from /assets route only
    let assets_dir = client_dist.join("assets");
    let assets_service = ServeDir::new(&assets_dir);

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/chains", get(list_chains).post(create_chain))
        .route("/api/chains/:id/start", post(start_chain))
        .route("/api/chains/:id/stop", post(stop_chain))
        .route("/api/chains/:id/restart", post(restart_chain))
        .route("/api/chains/:id/delete", post(delete_chain))
        .route("/api/chains/:id/logstream", get(log_stream))
        .route("/api/chains/:id/blockstream", get(block_stream))
        .route("/api/:chainid/:blocknumber", get(get_block))
        .nest_service("/assets", assets_service)
        .fallback(serve_static_or_index)
        .with_state(state);

    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();
    println!("listening on http://{}", addr);

    if let Err(err) = axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app).await {
        println!("server error {}", err.to_string());
    }
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

async fn serve_static_or_index(
    State(state): State<AppState>,
    req: axum::http::Request<axum::body::Body>,
) -> impl IntoResponse {
    use tower::ServiceExt;

    let path = req.uri().path();
    if path.ends_with(".js") || path.ends_with(".wasm") || path.ends_with(".css") {
        let static_service = tower_http::services::ServeDir::new(&state.client_dist);
        match static_service.oneshot(req).await {
            Ok(response) => {
                if response.status() != StatusCode::NOT_FOUND {
                    return response.into_response();
                }
            }
            Err(_) => {
                return (StatusCode::NOT_FOUND, "Not found").into_response();
            }
        }
    }

    // Otherwise, serve index.html for SPA routing
    let index_html = state.client_dist.join("index.html");
    match std::fs::read_to_string(index_html) {
        Ok(contents) => Html(contents).into_response(),
        Err(_) => (
            StatusCode::OK,
            Html("<html><body><h1>LocalChain</h1><p>Client not built yet. Run trunk build.</p></body></html>".to_string()),
        )
            .into_response(),
    }
}

async fn list_chains(State(state): State<AppState>) -> impl IntoResponse {
    let list = state.manager.list().await;
    Json(list)
}

async fn create_chain(
    State(state): State<AppState>,
    Json(req): Json<ChainConfig>,
) -> impl IntoResponse {
    match state.manager.create(req.clone()).await {
        Ok(_) => (StatusCode::OK, Json(req)).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

async fn start_chain(State(state): State<AppState>, Path(id): Path<u64>) -> impl IntoResponse {
    match state.manager.start(&id).await {
        Ok(()) => (StatusCode::OK).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

async fn stop_chain(State(state): State<AppState>, Path(id): Path<u64>) -> impl IntoResponse {
    match state.manager.stop(&id).await {
        Ok(()) => (StatusCode::OK).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

async fn restart_chain(State(state): State<AppState>, Path(id): Path<u64>) -> impl IntoResponse {
    match state.manager.restart(&id).await {
        Ok(()) => (StatusCode::OK).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

async fn delete_chain(State(state): State<AppState>, Path(id): Path<u64>) -> impl IntoResponse {
    match state.manager.delete(&id).await {
        Ok(()) => (StatusCode::OK).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

async fn log_stream(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Sse<impl Stream<Item = Result<sse::Event, Infallible>>> {
    let stream: Pin<Box<dyn Stream<Item = Result<sse::Event, Infallible>> + Send>> =
        match state.manager.subscribe_logs(&id).await {
            Ok(rx) => {
                let s = BroadcastStream::new(rx).map(|msg| match msg {
                    Ok(line) => Ok(sse::Event::default().data(line)),
                    Err(_) => Ok(sse::Event::default().event("ping").data("")),
                });
                Box::pin(s)
            }
            Err(_) => Box::pin(tokio_stream::once(Ok(sse::Event::default()
                .event("error")
                .data("not found")))),
        };
    Sse::new(stream).keep_alive(sse::KeepAlive::new())
}

async fn block_stream(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Sse<impl Stream<Item = Result<sse::Event, Infallible>>> {
    let stream: Pin<Box<dyn Stream<Item = Result<sse::Event, Infallible>> + Send>> =
        match state.manager.subscribe_blocks(&id).await {
            Ok(rx) => {
                let s = BroadcastStream::new(rx).map(|msg| match msg {
                    Ok(block) => Ok(sse::Event::default().data(block.to_json())),
                    Err(_) => Ok(sse::Event::default().event("ping").data("")),
                });
                Box::pin(s)
            }
            Err(_) => Box::pin(tokio_stream::once(Ok(sse::Event::default()
                .event("error")
                .data("not found")))),
        };
    Sse::new(stream).keep_alive(sse::KeepAlive::new())
}

async fn get_block(
    State(state): State<AppState>,
    Path((chain_id, block_number)): Path<(u64, u64)>,
) -> impl IntoResponse {
    match state.manager.get_block(&chain_id, block_number).await {
        Ok((block, transactions)) => (
            StatusCode::OK,
            Json(BlockResponse {
                block,
                transactions,
            }),
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}
