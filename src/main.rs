use std::{env, fmt::Pointer, fs, str};

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

    let lines_from_file: Vec<&str> = file_as_str.split('\n').collect();
    for ele in lines_from_file.iter() {
        println!("{}", ele);
    }
    let split_lines: Vec<Vec<&str>> = lines_from_file
        .iter()
        .map(|line| -> Vec<&str> { line.split('-').collect() })
        .collect();
    for line in split_lines.iter() {
        println!(
            "song name: {:?}, by the artist: {:?}",
            line.get(0).unwrap(),
            line.get(1).unwrap()
        );
    }
}
