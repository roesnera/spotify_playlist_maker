use rocket::serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Code {
    pub auth_code: Arc<Mutex<Option<String>>>,
    pub token: Arc<Mutex<Option<String>>>,
}

pub struct ClientInfo {
    pub id: Arc<Mutex<String>>,
    pub secret: Arc<Mutex<String>>,
}

#[derive(Deserialize)]
pub struct TokenReturn {
    pub access_token: String,
}

#[derive(Serialize)]
pub struct CreatePlaylistBody {
    pub name: String,
    pub description: String,
    pub public: bool,
}

#[derive(Deserialize)]
pub struct MeResponse {
    pub id: String,
    pub display_name: String,
}

#[derive(Deserialize)]
pub struct TrackResponse {
    pub tracks: TrackRequestData,
}

#[derive(Deserialize)]
pub struct TrackRequestData {
    pub items: Vec<Track>,
}

#[derive(Deserialize)]
pub struct Track {
    pub id: String,
}

#[derive(Deserialize)]
pub struct PlaylistResp {
    pub id: String,
}
