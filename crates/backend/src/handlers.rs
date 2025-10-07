use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::Value;
use tower_http::trace::TraceLayer;
use tracing::instrument;

use crate::error::AppError;
use crate::models::{
    CommandResponse, HealthResponse, ModeGetResponse, ModeSetRequest, MopidyHealth,
    PlaylistRequest, PlaylistResponse, SimilarQuery, SimilarResponse,
};
use crate::{discover, scripts, AppState};

pub fn app_routes(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/rpc", post(proxy_rpc))
        .route("/mode", get(get_mode).post(set_mode))
        .route("/playlists/from-list", post(playlist_from_list))
        .route("/discover/similar", get(discover_similar))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

#[instrument(skip(state))]
pub async fn health(State(state): State<AppState>) -> Result<Json<HealthResponse>, AppError> {
    let mut overall = "ok";
    let mopidy_status = if state.config.check_mopidy_health {
        match state.mopidy.health_check().await {
            Ok(_) => Some(MopidyHealth {
                status: "ok",
                detail: None,
            }),
            Err(err) => {
                overall = "degraded";
                Some(MopidyHealth {
                    status: "error",
                    detail: Some(err),
                })
            }
        }
    } else {
        None
    };

    Ok(Json(HealthResponse {
        status: overall,
        version: env!("CARGO_PKG_VERSION"),
        mopidy: mopidy_status,
    }))
}

#[instrument(skip(state, payload))]
pub async fn proxy_rpc(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let response = state.mopidy.proxy(payload).await?;
    Ok(Json(response))
}

#[instrument(skip(state))]
pub async fn get_mode(State(state): State<AppState>) -> Result<Json<ModeGetResponse>, AppError> {
    let output = scripts::show_mode(&state.config).await?;
    let trimmed = output.stdout.trim().to_string();
    let inferred = crate::models::AudioMode::infer(&trimmed);

    Ok(Json(ModeGetResponse {
        value: trimmed,
        mode: inferred,
    }))
}

#[instrument(skip(state, body))]
pub async fn set_mode(
    State(state): State<AppState>,
    Json(body): Json<ModeSetRequest>,
) -> Result<Json<CommandResponse>, AppError> {
    let output = scripts::set_mode(&state.config, &body).await?;
    Ok(Json(CommandResponse {
        stdout: output.stdout,
        stderr: output.stderr,
    }))
}

#[instrument(skip(state, body))]
pub async fn playlist_from_list(
    State(state): State<AppState>,
    Json(body): Json<PlaylistRequest>,
) -> Result<Json<PlaylistResponse>, AppError> {
    let output = scripts::playlist_from_list(&state.config, &body).await?;
    Ok(Json(PlaylistResponse {
        stdout: output.stdout,
        stderr: output.stderr,
    }))
}

#[instrument(skip(state, params))]
pub async fn discover_similar(
    State(state): State<AppState>,
    Query(params): Query<SimilarQuery>,
) -> Result<Json<SimilarResponse>, AppError> {
    let response =
        discover::similar_tracks(state.mopidy.as_ref(), &params.seed, params.limit).await?;

    Ok(Json(response))
}
