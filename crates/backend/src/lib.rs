pub mod config;
mod discover;
pub mod error;
mod handlers;
mod models;
mod mopidy;
mod scripts;
pub mod validation;

pub use error::AppError;
pub use models::AudioMode;
pub use mopidy::{HttpMopidyClient, MopidyClient};

use axum::Router;
use std::sync::Arc;

use crate::config::AppConfig;
use crate::handlers::app_routes;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub mopidy: Arc<dyn MopidyClient>,
}

pub fn build_router(config: AppConfig) -> Router {
    let config = Arc::new(config);
    let mopidy_client = Arc::new(HttpMopidyClient::new(
        reqwest::Client::new(),
        config.mopidy_rpc_url.clone(),
    )) as Arc<dyn MopidyClient>;

    app_routes(AppState {
        config,
        mopidy: mopidy_client,
    })
}

pub fn build_router_with_mopidy(config: AppConfig, mopidy_client: Arc<dyn MopidyClient>) -> Router {
    app_routes(AppState {
        config: Arc::new(config),
        mopidy: mopidy_client,
    })
}
