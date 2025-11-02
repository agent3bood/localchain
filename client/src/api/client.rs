use gloo_net::{eventsource::futures::EventSource, http::Request};
use once_cell::sync::OnceCell;
use shared::types::chain_config::ChainConfig;
use std::sync::Arc;

static INSTANCE: OnceCell<Arc<Api>> = OnceCell::new();

pub struct Api {
    base_url: String,
}

impl Api {
    pub fn init(base_url: String) {
        let _ = INSTANCE.set(Arc::new(Api { base_url }));
    }

    pub fn instance() -> Arc<Self> {
        INSTANCE.get().unwrap().clone()
    }

    pub async fn list_chains(&self) -> Result<Vec<ChainConfig>, String> {
        let resp = Request::get(format!("{}/api/chains", self.base_url).as_str())
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !resp.ok() {
            return Err(format!(""));
        }
        resp.json().await.map_err(|e| e.to_string())
    }

    pub async fn create_chain(&self, config: &ChainConfig) -> Result<ChainConfig, String> {
        let resp = Request::post(format!("{}/api/chains", self.base_url).as_str())
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

    pub async fn post_action(&self, chain_id: &u64, action: &str) -> Result<(), String> {
        let url = format!("{}/api/chains/{}/{}", self.base_url, chain_id, action);
        let resp = Request::post(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !resp.ok() {
            return Err(format!("HTTP {}", resp.status()));
        }
        Ok(())
    }

    pub fn log_stream(&self, id: u64) -> Result<EventSource, String> {
        let url = format!("/api/chains/{}/logstream", id);
        EventSource::new(&url).map_err(|e| format!("{e:?}"))
    }
}
