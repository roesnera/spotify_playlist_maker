use reqwest::{self, Body};
use rocket::{
    form::validate::Len,
    http::Status,
    response::status::Custom,
    serde::json::{serde_json::json, Value},
};

use crate::models::*;

pub fn not_found_error() -> Custom<Value> {
    Custom(Status::NotFound, json!("Error: not found!"))
}

pub async fn get_spotify_id<'a>(
    token: &'a str,
    song_name: &'a str,
    artist_name: &'a str,
) -> Result<String, Error> {
    let client = reqwest::Client::new();
    let url: &str = "https://api.spotify.com/v1/search";
    let request = client.get(url).bearer_auth(token).query(&[
        ["q", &*format!("track:{} artist:{}", song_name, artist_name)],
        ["type", "track"],
    ]);

    let response = match request.send().await {
        Result::Ok(response) => match response.json::<TrackResponse>().await {
            Ok(response) => response,
            _ => return Err(Error::TrackNotFound),
        },
        Result::Err(e) => panic!("{}", e),
    };
    let tracks = response.tracks;
    let items = tracks.items;
    let item = match items.get(0) {
        Some(item) => item,
        None => return Err(Error::IdNotFound),
    };
    Ok(item.id.to_owned())
}
pub async fn get_token_auth_code(
    client_id: &str,
    client_secret: &str,
    code: &str,
) -> Result<String, Error> {
    let client = reqwest::Client::new();
    let url: &str = "https://accounts.spotify.com/api/token";
    let body_str: String = format!(
        "grant_type=authorization_code&redirect_uri=http://127.0.0.1:8000/success&code={}",
        code
    );

    let request = client
        .post(url)
        .basic_auth(client_id, Some(client_secret))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(body_str));

    let response = match request.send().await {
        Ok(resp) => match resp.json::<TokenReturn>().await {
            Ok(token) => token,
            _ => return Err(Error::NoAccess),
        },
        _ => return Err(Error::BadRequest),
    };
    Ok(response.access_token.to_owned())
}

pub async fn get_token(client_id: &str, client_secret: &str) -> Result<String, Error> {
    let client = reqwest::Client::new();
    let url: &str = "https://accounts.spotify.com/api/token";
    let body_str: String = format!(
        "grant_type=client_credentials&client_id={}&client_secret={}",
        client_id, client_secret
    );
    let request = client
        .post(url)
        .header("Content-type", "application/x-www-form-urlencoded")
        .body(Body::from(body_str));

    let response = match request.send().await {
        Result::Ok(resp) => match resp.json::<TokenReturn>().await {
            Ok(response) => response,
            _ => return Err(Error::TokenNotFound),
        },
        _ => panic!("bad token request!"),
    };
    Ok(response.access_token.to_string())
}

pub async fn get_me(token: &str) -> Result<String, Error> {
    let client = reqwest::Client::new();

    let request = client
        .get("https://api.spotify.com/v1/me")
        .bearer_auth(token);

    let response = match request.send().await {
        Result::Ok(some) => match some.json::<MeResponse>().await {
            Ok(response) => response,
            _ => return Err(Error::MeNotFound),
        },
        _ => panic!("bad me request"),
    };
    Ok(response.id.to_owned())
}

pub async fn add_to_playlist(
    song_ids: &Vec<String>,
    playlist_id: &str,
    token: &str,
) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();

    let url = format!(
        "https://api.spotify.com/v1/playlists/{}/tracks",
        playlist_id
    );

    let query_string: Vec<String> = song_ids
        .iter()
        .map(|id| format!("spotify:track:{}", id))
        .collect();
    for i in 0..query_string.len() {
        let body = AddPlaylistSongsBody {
            uris: vec![query_string
                .get(i)
                .unwrap_or(&String::from("missing"))
                .to_string()],
        };

        // let query_string = &query_string.join(",");
        // println!("{:?}", query_string);

        let request = client
            .post(url.clone())
            .bearer_auth(token)
            // .query(&[("uris", query_string)])
            .json(&body);

        let _ = match request.send().await {
            Ok(resp) => resp,
            Err(e) => return Err(e),
        };
    }

    Ok("Added playlists successfully".to_string())
}
pub async fn create_playlist(
    token: &str,
    name: &str,
    desc: &str,
    user_id: &str,
) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let url: String = format!("https://api.spotify.com/v1/users/{}/playlists", user_id);
    let body = CreatePlaylistBody {
        name: name.to_owned(),
        description: desc.to_owned(),
        public: true,
    };

    let request: reqwest::RequestBuilder = client
        .post(url)
        .bearer_auth(token)
        .header("Content-type", "application/json")
        .json(&body);

    let response = match request.send().await {
        Result::Ok(resp) => match resp.json::<PlaylistResp>().await {
            Ok(response) => response,
            Err(e) => return Err(e),
        },
        _ => panic!("bad playlist create request"),
    };

    Ok(response.id.to_owned())
}

pub async fn check_for_playlist(token: &str, name: &str, user_id: &str) -> Result<String, Error> {
    let client = reqwest::Client::new();
    let url: String = format!("https://api.spotify.com/v1/users/{}/playlists", *&user_id).into();
    let request = client.get(url).bearer_auth(token);

    let response: AllPlaylistsResponse = match request.send().await {
        Ok(response) => match response.json::<AllPlaylistsResponse>().await {
            Ok(playlistResponse) => playlistResponse,
            _ => return Err(Error::UserDataNotFound),
        },
        _ => return Err(Error::BadRequest),
    };

    let playlist_id = match response
        .items
        .iter()
        .find(|playlist| playlist.name.eq(name))
    {
        Some(playlist) => Ok(playlist.id.clone()),
        _ => Err(Error::PlaylistsNotFound),
    };
    playlist_id
}
