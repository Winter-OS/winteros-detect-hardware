use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde_json;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

fn request_forbiden_fix(
    client: &Client,
    mut response: reqwest::blocking::Response,
    url: &str,
) -> reqwest::blocking::Response {
    while response.status() == StatusCode::FORBIDDEN {
        if let Some(reset_time_header) = response.headers().get("X-RateLimit-Reset") {
            println!("{:?}", reset_time_header);
            if let Ok(reset_time) = reset_time_header.to_str().unwrap_or("0").parse::<u64>() {
                let current_time = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                    Ok(duration) => duration,
                    Err(err) => panic!("{}", err.to_string()),
                }
                .as_secs();
                let sleep_time = reset_time.saturating_sub(current_time);
                println!("Rate limited. Sleeping for {} seconds.", sleep_time);
                sleep(Duration::from_secs(sleep_time));
            }
        }
        response = client.get(url).send().map_err(|e| e.to_string()).unwrap();
    }
    response
}

fn get_sha(client: &Client, author: &str, repo: &str) -> Result<String, String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/branches/master",
        author, repo
    );

    let mut response = match client
        .get(&url)
        .header("Accept", "application/json")
        .header("User-Agent", "Rust")
        .send()
    {
        Ok(resp) => resp,
        Err(err) => return Err(err.to_string()),
    };

    response = request_forbiden_fix(&client, response, &url);

    if response.status() != StatusCode::OK {
        return Err(format!(
            "Invalid author or repo name: {}",
            response.status()
        ));
    }
    let json: serde_json::Value = response
        .text()
        .expect("error")
        .parse()
        .map_err(|e: serde_json::Error| e.to_string())
        .unwrap();
    json["commit"]["commit"]["tree"]["sha"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| String::from("Unexpcted response format"))
}

pub fn get_git_tree(author: &str, repo: &str) -> Result<serde_json::Value, String> {
    let client = Client::new();

    let sha = match get_sha(&client, author, repo) {
        Ok(sha) => sha,
        Err(err) => return Err(err),
    };

    let url = format!(
        "https://api.github.com/repos/{}/{}/git/trees/{}?recursive=1",
        author, repo, sha
    );

    let mut response = match client
        .get(&url)
        .header("Accept", "application/json")
        .header("User-Agent", "Rust")
        .send()
    {
        Ok(resp) => resp,
        Err(err) => return Err(err.to_string()),
    };

    response = request_forbiden_fix(&client, response, &url);

    if response.status() != StatusCode::OK {
        return Err(format!("Failed to get git tree: {}", response.status()));
    }
    let json: serde_json::Value = response
        .text()
        .expect("Error during text convertion of request")
        .parse()
        .map_err(|e: serde_json::Error| e.to_string())
        .unwrap();
    println!("{:#?}", json);

    match json.get("tree") {
        Some(value) => Ok(value.clone()),
        None => Err(String::from("Unexpcted response format")),
    }
}
