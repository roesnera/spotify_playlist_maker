use std::{env, fs, str};

use regex::{Regex, Split};

fn main() {
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

    let spotify_ids: Vec<&str> = split_lines
        .iter()
        .map(|line_vec| -> &str { get_spotify_id(line_vec.get(0).expect("missing song name!")) })
        .collect();

    for id in spotify_ids.iter() {
        println!("{}", id);
    }
}

fn get_spotify_id(song_name: &str) -> &str {
    println!("retreiving spotify id for song: {:?}", song_name);
    "some id"
}
