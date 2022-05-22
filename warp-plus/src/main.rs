use std::time;
use std::env;
use std::{collections::HashMap, thread::sleep};

use rand::Rng;

const LETTERS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const DIGITS: &str = "0123456789";

fn main() -> ! {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <your ID>.", &args[0]);
        std::process::exit(1);
    }

    let id = &args[1];

    loop {
        let url = format!(
            "https://api.cloudflareclient.com/v0a{}/reg",
            gen_string(3, DIGITS)
        );

        let install_id = gen_string(22, LETTERS);

        let key = format!("{}=", gen_string(43, LETTERS));
        let fcm_token = format!("{}:APA91b{}", install_id, gen_string(134, LETTERS));

        let mut body = HashMap::new();
        body.insert("key", key.as_str());
        body.insert("install_id", install_id.as_str());
        body.insert("fcm_token", fcm_token.as_str());
        body.insert("referrer", id);
        body.insert("type", "Android");
        body.insert("locale", "es_ES");

        let request = reqwest::blocking::Client::new();
        match request
            .post(url)
            .header("Content-Type", "application/json; charset=UTF-8")
            .header("Host", "api.cloudflareclient.com")
            .header("Connection", "Keep-Alive")
            .header("Accept-Encoding", "gzip")
            .header("User-Agent", "okhttp/3.12.1")
            .json(&body)
            .send()
        {
            Ok(response) => {
                if response.status().is_success() {
                    println!("1GB has been added to the account");
                } else {
                    println!("Request failed with HTTP status: {:?}", response.status());
                }
            }
            Err(e) => {
                println!("Error {:#?}", e);
            }
        };

        sleep(time::Duration::new(20, 0));
    }
}

fn gen_string(length: u32, string: &str) -> String {
    let mut digit_str = String::new();
    let mut rng = rand::thread_rng();
    for _ in 0..length {
        digit_str.push(string.chars().nth(rng.gen_range(0..string.len())).unwrap());
    }

    digit_str
}
