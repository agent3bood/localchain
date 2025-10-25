use std::{collections::HashMap, net::SocketAddr, path::PathBuf, sync::Arc};

use crate::anvil::process::AnvilProcess;

mod anvil;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{sse, Html, IntoResponse, Sse},
    routing::{get, post},
    Json, Router,
};
use futures::Stream;
use shared::{ChainConfig, ChainStatus};
use std::convert::Infallible;
use std::pin::Pin;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    sync::{broadcast, Mutex},
};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tower_http::services::ServeDir;

#[derive(Clone)]
struct AppState {
    client_dist: PathBuf,
    manager: Arc<ChainsManager>,
}

struct ChainEntry {
    id: String,
    config: ChainConfig,
    log_tx: Arc<broadcast::Sender<String>>,
    process: AnvilProcess,
}

#[derive(Default)]
struct ChainsManager {
    inner: Mutex<HashMap<String, ChainEntry>>, // key: chain id (name)
}

impl ChainsManager {
    async fn list(&self) -> Vec<ChainConfig> {
        let map = self.inner.lock().await;
        map.values().map(|c| c.config.clone()).collect()
    }

    async fn create(&self, cfg: ChainConfig) -> Result<String, String> {
        let mut map = self.inner.lock().await;
        if map.contains_key(&cfg.name) {
            return Err("name already exists".into());
        }
        let (tx, _rx) = broadcast::channel(1024);
        let log_tx = Arc::new(tx);
        let process = AnvilProcess::new(
            cfg.name.clone(),
            cfg.chain_id,
            cfg.port,
            cfg.block_time,
            log_tx.clone(),
        );
        let entry = ChainEntry {
            id: cfg.name.clone(),
            config: cfg,
            log_tx: log_tx,
            process: process,
        };
        let id = entry.id.clone();
        map.insert(id.clone(), entry);
        drop(map);
        Ok(id)
    }

    async fn start(&self, id: &str) -> Result<(), String> {
        let mut map = self.inner.lock().await;
        let Some(entry) = map.get_mut(id) else {
            return Err("not found".into());
        };
        entry.config.status = ChainStatus::Starting;
        match entry.process.start().await {
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

    async fn stop(&self, id: &str) -> Result<(), String> {
        let mut map = self.inner.lock().await;
        let Some(entry) = map.get_mut(id) else {
            return Err("not found".into());
        };
        entry.process.stop().await?;
        entry.config.status = ChainStatus::Stopped;
        let _ = entry.log_tx.send("[manager] stopped".into());
        Ok(())
    }

    async fn restart(&self, id: &str) -> Result<(), String> {
        self.stop(id).await?;
        self.start(id).await
    }

    async fn subscribe_logs(&self, id: &str) -> Result<broadcast::Receiver<String>, String> {
        let map = self.inner.lock().await;
        let Some(entry) = map.get(id) else {
            return Err("not found".into());
        };
        Ok(entry.log_tx.subscribe())
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

    let static_service = ServeDir::new(&client_dist);

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/chains", get(list_chains).post(create_chain))
        .route("/api/chains/:id/start", post(start_chain))
        .route("/api/chains/:id/stop", post(stop_chain))
        .route("/api/chains/:id/restart", post(restart_chain))
        .route("/api/chains/:id/logstream", get(log_stream))
        // index route serves the built client index.html
        .route("/", get(index))
        // serve other client assets under /
        .fallback_service(static_service)
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

async fn index(State(state): State<AppState>) -> impl IntoResponse {
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

async fn start_chain(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.manager.start(&id).await {
        Ok(()) => (StatusCode::OK).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

async fn stop_chain(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.manager.stop(&id).await {
        Ok(()) => (StatusCode::OK).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

async fn restart_chain(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.manager.restart(&id).await {
        Ok(()) => (StatusCode::OK).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

async fn log_stream(
    State(state): State<AppState>,
    Path(id): Path<String>,
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
