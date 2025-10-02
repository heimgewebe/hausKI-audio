pub mod config;
mod error;
mod handlers;
mod mopidy;
mod scripts;
mod models;
mod discover;

use axum::Router;
use std::sync::Arc;

use crate::config::AppConfig;
use crate::handlers::app_routes;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub http_client: reqwest::Client,
}

pub fn build_router(config: AppConfig) -> Router {
    let state = AppState {
        config: Arc::new(config),
        http_client: reqwest::Client::new(),
    };

    app_routes(state)
}
