use reqwest::{self, Body, Result};
use serde::Deserialize;

#[derive(Deserialize)]
struct TokenReturn {
    access_token: String,
}

#[derive(Deserialize)]
struct RequestReturn {
    tracks: TrackRequestData,
}

#[derive(Deserialize)]
struct TrackRequestData {
    items: Vec<Track>,
}

#[derive(Deserialize)]
struct Track {
    id: String,
}

pub async fn get_spotify_id<'a>(
    token: &'a str,
    song_name: &'a str,
    artist_name: &'a str,
) -> Result<String> {
    let client = reqwest::Client::new();
    let url: &str = "https://api.spotify.com/v1/search";
    let request = client.get(url).bearer_auth(token).query(&[
        ["q", &*format!("track:{} artist:{}", song_name, artist_name)],
        ["type", "track"],
    ]);
    println!("retreiving spotify id for song: {:?}", song_name);

    let response = match request.send().await {
        Result::Ok(response) => response.json::<RequestReturn>().await?,
        Result::Err(e) => panic!("{}", e),
    };
    let tracks = response.tracks;
    let items = tracks.items;
    let item = items.get(0).expect("missing item id!");
    Ok(item.id.to_owned())
}

pub async fn get_token(client_id: &str, client_secret: &str) -> Result<String> {
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
        Result::Ok(resp) => resp.json::<TokenReturn>().await?,
        _ => panic!("bad token request!"),
    };
    Ok(response.access_token.to_string())
}
