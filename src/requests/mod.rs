use reqwest::{self, Body};
use rocket::{
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
    println!("retreiving spotify id for song: {:?}", song_name);

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
        Result::Ok(some) => match some.text().await {
            Ok(response) => response,
            _ => return Err(Error::MeNotFound),
        },
        _ => panic!("bad me request"),
    };
    Ok(response)
}

pub async fn create_playlist(token: &str, name: &str, desc: &str) -> Result<String, Error> {
    let client = reqwest::Client::new();
    let url: &str = "https://api.spotify.com/v1/";
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
            _ => return Err(Error::Rejected),
        },
        _ => panic!("bad playlist create request"),
    };

    Ok(response.id.to_owned())
}
