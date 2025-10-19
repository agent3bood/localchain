use std::{net::SocketAddr, path::PathBuf};

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use tower_http::services::ServeDir;
use tracing::{error, info};

#[derive(Clone)]
struct AppState {
    client_dist: PathBuf,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .compact()
        .init();

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
    };

    let static_service = ServeDir::new(&client_dist);

    let app = Router::new()
        .route("/api/health", get(health))
        // index route serves the built client index.html
        .route("/", get(index))
        // serve other client assets under /
        .fallback_service(static_service)
        .with_state(state);

    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();
    info!("listening on http://{}", addr);

    if let Err(err) = axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app).await {
        error!(?err, "server error");
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
