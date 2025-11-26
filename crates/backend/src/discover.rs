use std::collections::HashSet;

use serde_json::Value;
use tracing::instrument;

use crate::error::AppError;
use crate::models::{SimilarResponse, SimilarTrack};
use crate::mopidy::MopidyClient;

#[instrument(skip(mopidy))]
pub async fn similar_tracks(
    mopidy: &dyn MopidyClient,
    seed: &str,
    limit: Option<usize>,
) -> Result<SimilarResponse, AppError> {
    let seed_track_value = mopidy.lookup_track(seed).await?;
    let seed_track_value =
        seed_track_value.ok_or_else(|| AppError::bad_request("seed track not found in Mopidy"))?;

    let seed_track = build_track(&seed_track_value)
        .ok_or_else(|| AppError::internal("seed track missing uri"))?;

    let query = build_query(&seed_track_value)
        .ok_or_else(|| AppError::internal("unable to derive search query from seed track"))?;

    let target_limit = limit.unwrap_or(10);
    if target_limit == 0 {
        return Ok(SimilarResponse {
            seed: seed_track,
            query,
            tracks: Vec::new(),
        });
    }

    let search_results = mopidy.search_any(&query).await?;
    let mut seen: HashSet<String> = HashSet::new();
    seen.insert(seed_track.uri.clone());
    let mut collected: Vec<SimilarTrack> = Vec::new();

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

    // Name MUSS existieren und nicht leer sein
    let name = track
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())?
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use serde_json::json;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct StubMopidy {
        lookup: Option<Value>,
        search: Vec<Value>,
        queries: Arc<Mutex<Vec<String>>>,
    }

    impl StubMopidy {
        fn new(lookup: Option<Value>, search: Vec<Value>) -> Self {
            Self {
                lookup,
                search,
                queries: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl MopidyClient for StubMopidy {
        async fn proxy(&self, _payload: Value) -> Result<Value, AppError> {
            unreachable!("proxy should not be invoked directly in tests");
        }

        async fn lookup_track(&self, _uri: &str) -> Result<Option<Value>, AppError> {
            Ok(self.lookup.clone())
        }

        async fn search_any(&self, query: &str) -> Result<Vec<Value>, AppError> {
            self.queries.lock().unwrap().push(query.to_string());
            Ok(self.search.clone())
        }
    }

    #[tokio::test]
    async fn similar_tracks_returns_empty_when_limit_is_zero() {
        let seed = json!({
            "uri": "qobuz:track:seed",
            "name": "Seed",
            "artists": [{"name": "Artist"}]
        });
        let mopidy = StubMopidy::new(Some(seed), vec![json!({"tracks": []})]);

        let response = similar_tracks(&mopidy, "qobuz:track:seed", Some(0))
            .await
            .expect("response");

        assert!(response.tracks.is_empty());
        assert_eq!(response.query, "Artist Seed");
        // With limit=0, search_any should NOT be called (performance optimization)
        let recorded = mopidy.queries.lock().unwrap().clone();
        assert!(
            recorded.is_empty(),
            "search_any should not be called when limit is 0"
        );
    }

    #[tokio::test]
    async fn similar_tracks_skips_seed_and_duplicates() {
        let seed_track = json!({
            "uri": "qobuz:track:seed",
            "name": "Seed",
            "artists": [{"name": "Artist"}]
        });
        let results = json!({
            "tracks": [
                {
                    "uri": "qobuz:track:seed",
                    "name": "Seed",
                    "artists": [{"name": "Artist"}]
                },
                {
                    "uri": "qobuz:track:1",
                    "name": "Track One",
                    "artists": [{"name": "Artist"}]
                },
                {
                    "uri": "qobuz:track:1",
                    "name": "Track One",
                    "artists": [{"name": "Artist"}]
                },
                {
                    "uri": "qobuz:track:2",
                    "name": "Track Two",
                    "artists": [{"name": "Artist"}],
                    "album": {"name": "Album"}
                }
            ]
        });
        let mopidy = StubMopidy::new(Some(seed_track), vec![results]);

        let response = similar_tracks(&mopidy, "qobuz:track:seed", Some(10))
            .await
            .expect("response");

        let uris: Vec<_> = response
            .tracks
            .iter()
            .map(|track| track.uri.as_str())
            .collect();
        assert_eq!(uris, vec!["qobuz:track:1", "qobuz:track:2"]);
        assert_eq!(response.tracks[1].album.as_deref(), Some("Album"));
    }
}
