use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum AudioMode {
    Pulse,
    Alsa,
}

impl AudioMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            AudioMode::Pulse => "pulse",
            AudioMode::Alsa => "alsa",
        }
    }

    pub fn infer(raw: &str) -> Option<Self> {
        let normalized = raw.to_ascii_lowercase();
        if normalized.contains("alsa") {
            Some(AudioMode::Alsa)
        } else if normalized.contains("pulse") {
            Some(AudioMode::Pulse)
        } else {
            None
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ModeSetRequest {
    pub mode: AudioMode,
}

#[derive(Debug, Serialize)]
pub struct ModeGetResponse {
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<AudioMode>,
}

#[derive(Debug, Serialize)]
pub struct CommandResponse {
    pub stdout: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub stderr: String,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistRequest {
    pub name: String,
    pub uris: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PlaylistResponse {
    pub stdout: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub stderr: String,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mopidy: Option<MopidyHealth>,
}

#[derive(Debug, Serialize)]
pub struct MopidyHealth {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SimilarQuery {
    pub seed: String,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SimilarTrack {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub artists: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SimilarResponse {
    pub seed: SimilarTrack,
    pub query: String,
    pub tracks: Vec<SimilarTrack>,
}

