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
pub async fn success(code: &str, auth_state: &State<Code>) -> Redirect {
    let mut code_state = auth_state.auth_code.lock().await;
    *code_state = Some(code.to_owned());
    Redirect::to(uri!(make_playlist()))
}

#[get("/makePlaylist")]
pub async fn make_playlist(
    auth_state: &State<Code>,
    client_info: &State<ClientInfo>,
) -> Option<NamedFile> {
    let mut token = auth_state.token.lock().await;
    let client_id = client_info.id.lock().await;
    let client_secret = client_info.secret.lock().await;
    let retrieved_token = get_token(&client_id, &client_secret)
        .await
        .expect("unable to retrieve token");
    *token = Some(retrieved_token);
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
    let retrieved_token = get_token(&client_id, &client_secret)
        .await
        .expect("unable to retrieve token");

    let code = code_state.auth_code.lock().await;

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
            let file_as_str = match str::from_utf8(&text) {
                Ok(v) => v.to_string(),
                Err(_) => panic!("Invalid utf 8"),
            };
            println!("matched!");
            println!("{:?}", file_as_str);

            let lines_from_file: Vec<&str> =
                file_as_str.split('\n').filter(|s| s.len() > 0).collect();
            for ele in lines_from_file.iter() {
                println!("{}", ele);
            }
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
                    &retrieved_token,
                    line.get(0).expect("no song here!"),
                    line.get(1).expect("no artist here!"),
                )
                .await
                .unwrap_or_else(|_| "none".to_string());
                if !(id_result.eq("none")) {
                    spotify_ids.push(id_result);
                }
            }

            for id in spotify_ids.iter() {
                println!("{}", id);
            }

            Result::Ok("file opened".to_string())
        }
        None => Result::Err("unable to open file".to_string()),
    }
}
