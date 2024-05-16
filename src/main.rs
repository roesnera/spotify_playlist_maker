use dotenv::dotenv;
use regex::Regex;
use rocket::{data::Data, fs::NamedFile, http::ContentType, response::Redirect, State};
use rocket_multipart_form_data::{
    mime, MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};
use std::{env, path::Path, str};
use tokio::sync::Mutex;
#[macro_use]
extern crate rocket;

pub mod requests;
use requests::*;

pub mod models;
use models::*;

#[launch]
fn rocket() -> _ {
    dotenv().ok();

    let client_id = env::var("CLIENT_ID").expect("Must set CLIENT_ID env variable");
    let client_secret = env::var("CLIENT_SECRET").expect("Must set CLIENT_SECRET env variable");

    rocket::build()
        .manage(Code {
            auth_code: Mutex::new(None).into(),
            token: Mutex::new(None).into(),
            user_id: Mutex::new(None).into(),
        })
        .manage(ClientInfo {
            id: Mutex::new(client_id).into(),
            secret: Mutex::new(client_secret).into(),
        })
        .mount("/", routes![index, success, make_playlist, send_file])
}

#[get("/")]
pub async fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new("static/index.html")).await.ok()
}

#[get("/success?<code>")]
pub async fn success(
    code: &str,
    auth_state: &State<Code>,
    client_info: &State<ClientInfo>,
) -> Redirect {
    let mut code_state = auth_state.auth_code.lock().await;
    let mut token_state = auth_state.token.lock().await;

    let authorization_token = get_token_auth_code(
        &client_info.id.lock().await.as_ref(),
        &client_info.secret.lock().await.as_ref(),
        code,
    )
    .await;
    *token_state = Some(authorization_token.unwrap());
    *code_state = Some(code.to_owned());
    Redirect::to(uri!(make_playlist()))
}

#[get("/makePlaylist")]
pub async fn make_playlist(auth_state: &State<Code>) -> Option<NamedFile> {
    let mut user_id = auth_state.user_id.lock().await;
    println!("Retrieved auth code token");
    let retrieved_user_id = get_me(auth_state.token.lock().await.as_ref().unwrap())
        .await
        .unwrap();
    *user_id = Some(retrieved_user_id);
    NamedFile::open(Path::new("static/makePlaylist.html"))
        .await
        .ok()
}

#[post("/makePlaylist", data = "<data>")]
pub async fn send_file(
    content_type: &ContentType,
    data: Data<'_>,
    code_state: &State<Code>,
    client_info: &State<ClientInfo>,
) -> Result<String, String> {
    let client_id = client_info.id.lock().await;
    let client_secret = client_info.secret.lock().await;

    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::raw("file")
            .content_type_by_string(Some(mime::TEXT_STAR))
            .unwrap(),
    ]);

    let mut multipart_form_data = match MultipartFormData::parse(content_type, data, options).await
    {
        Ok(r) => r,
        Err(e) => return Result::Err("unable to parse file".to_string()),
    };

    match multipart_form_data.raw.remove("file") {
        Some(mut files) => {
            let file = files.remove(0);

            let text = file.raw;
            let name = file.file_name.unwrap();
            let file_as_str = match str::from_utf8(&text) {
                Ok(v) => v.to_string(),
                Err(_) => panic!("Invalid utf 8"),
            };
            println!("matched!");

            let lines_from_file: Vec<&str> =
                file_as_str.split('\n').filter(|s| s.len() > 0).collect();
            let split_lines: Vec<Vec<&str>> = lines_from_file
                .iter()
                .map(|line| -> Vec<&str> {
                    let re = Regex::new(r"by|-").expect("Invalid regex!");
                    re.split(line).map(|item| -> &str { item.trim() }).collect()
                })
                .collect();
            for line in split_lines.iter() {
                if line.get(0).is_some() && line.get(1).is_some() {
                    println!(
                        "song name: {:?}, by the artist: {:?}",
                        line.get(0).unwrap(),
                        line.get(1).unwrap()
                    );
                }
            }

            let mut spotify_ids = Vec::new();

            for line in split_lines.iter() {
                let id_result = get_spotify_id(
                    &code_state.token.lock().await.as_ref().unwrap(),
                    line.get(0).expect("no song here!"),
                    line.get(1).expect("no artist here!"),
                )
                .await
                .unwrap_or_else(|_| "none".to_string());
                if !(id_result.eq("none")) {
                    spotify_ids.push(id_result);
                }
            }
            let token = code_state.token.lock().await.as_ref().unwrap().clone();
            let user_id = code_state.user_id.lock().await.as_ref().unwrap().clone();

            println!("making playlist");
            println!("token: {:?}", token);
            println!("user_id: {:?}", user_id);
            let playlist_id = match check_for_playlist(&token, &name, &user_id).await {
                Ok(id) => id,
                _ => create_playlist(&token, &name, "a new playlist", &user_id)
                    .await
                    .unwrap(),
            };
            println!("Playlist id:");
            println!("{}", playlist_id);

            let playlist_snapshot_id = add_to_playlist(&spotify_ids, &playlist_id, &token)
                .await
                .unwrap();

            println!("{:?}", playlist_snapshot_id);

            Result::Ok("playlist created and songs added".to_owned())
        }
        None => Result::Err("unable to open file".to_string()),
    }
}
