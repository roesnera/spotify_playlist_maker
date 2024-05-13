use dotenv::dotenv;
use regex::Regex;
use std::{env, fs, str};

pub mod requests;
use requests::*;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let client_id = env::var("CLIENT_ID").expect("Must set CLIENT_ID env variable");
    let client_secret = env::var("CLIENT_SECRET").expect("Must set CLIENT_SECRET env variable");

    let token = get_token(&client_id, &client_secret).await.unwrap();
    println!("{}", token);

    let my_id: String = get_me(&token).await.unwrap();
    println!("{}", my_id);

    let args: Vec<String> = env::args().collect::<Vec<String>>();

    let filename: String = args.get(1).unwrap().to_string();
    dbg!(filename.clone());

    let file_buff: Vec<u8> = fs::read(filename).unwrap();
    let file_as_str = match str::from_utf8(&file_buff) {
        Ok(v) => v.to_string(),
        Err(_) => panic!("Invalid utf 8"),
    };
    println!("matched!");
    println!("{:?}", file_as_str);

    let lines_from_file: Vec<&str> = file_as_str.split('\n').filter(|s| s.len() > 0).collect();
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
        let id = get_spotify_id(
            &token,
            line.get(0).expect("no song here!"),
            line.get(1).expect("no artist here!"),
        )
        .await
        .expect("Id unavailable!");
        spotify_ids.push(id);
    }

    for id in spotify_ids.iter() {
        println!("{}", id);
    }
}
