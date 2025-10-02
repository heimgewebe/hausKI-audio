use std::collections::HashSet;

use serde_json::Value;
use tracing::instrument;

use crate::error::AppError;
use crate::models::{SimilarResponse, SimilarTrack};
use crate::mopidy;

#[instrument(skip(client, url))]
pub async fn similar_tracks(
    client: &reqwest::Client,
    url: &url::Url,
    seed: &str,
    limit: Option<usize>,
) -> Result<SimilarResponse, AppError> {
    let seed_track_value = mopidy::lookup_track(client, url, seed).await?;
    let seed_track_value = seed_track_value
        .ok_or_else(|| AppError::bad_request("seed track not found in Mopidy"))?;

    let seed_track = build_track(&seed_track_value)
        .ok_or_else(|| AppError::internal("seed track missing uri"))?;

    let query = build_query(&seed_track_value).ok_or_else(|| {
        AppError::internal("unable to derive search query from seed track")
    })?;

    let search_results = mopidy::search_any(client, url, &query).await?;
    let mut seen: HashSet<String> = HashSet::new();
    seen.insert(seed_track.uri.clone());
    let mut collected: Vec<SimilarTrack> = Vec::new();

    let target_limit = limit.unwrap_or(10);
    if target_limit == 0 {
        return Ok(SimilarResponse {
            seed: seed_track,
            query,
            tracks: collected,
        });
    }

    for backend in search_results {
        if let Some(tracks) = backend.get("tracks").and_then(Value::as_array) {
            for track in tracks {
                let Some(candidate) = build_track(track) else {
                    continue;
                };
                if !seen.insert(candidate.uri.clone()) {
                    continue;
                }
                collected.push(candidate);
                if collected.len() >= target_limit {
                    break;
                }
            }
        }
        if collected.len() >= target_limit {
            break;
        }
    }

    Ok(SimilarResponse {
        seed: seed_track,
        query,
        tracks: collected,
    })
}

fn build_query(track: &Value) -> Option<String> {
    let name = track.get("name").and_then(Value::as_str)?.trim();
    if name.is_empty() {
        return None;
    }

    let artists = track
        .get("artists")
        .and_then(Value::as_array)
        .and_then(|arr| arr.first())
        .and_then(|artist| artist.get("name"))
        .and_then(Value::as_str)
        .map(|s| s.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());

    let query = if let Some(artist) = artists {
        format!("{artist} {name}")
    } else {
        name.to_string()
    };

    Some(query)
}

fn build_track(track: &Value) -> Option<SimilarTrack> {
    let uri = track.get("uri").and_then(Value::as_str)?.to_string();
    let name = track
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let album = track
        .get("album")
        .and_then(|album| album.get("name"))
        .and_then(Value::as_str)
        .map(|s| s.to_string());
    let artists = track
        .get("artists")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|artist| artist.get("name").and_then(Value::as_str))
                .map(|name| name.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Some(SimilarTrack {
        uri,
        name,
        album,
        artists,
    })
}
