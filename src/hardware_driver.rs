use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde_json;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

#[derive(Debug)]
pub struct HardwareModule {
    module_computer: Vec<String>,
    module_common: Vec<String>,
}

impl HardwareModule {
    fn request_forbiden_fix(
        client: &Client,
        mut response: reqwest::blocking::Response,
        url: &str,
    ) -> reqwest::blocking::Response {
        while response.status() == StatusCode::FORBIDDEN {
            if let Some(reset_time_header) = response.headers().get("X-RateLimit-Reset") {
                if let Ok(reset_time) = reset_time_header.to_str().unwrap_or("0").parse::<u64>() {
                    let current_time =
                        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
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
            .header("User-Agent", "WinterOS")
            .send()
        {
            Ok(resp) => resp,
            Err(err) => return Err(err.to_string()),
        };

        response = Self::request_forbiden_fix(&client, response, &url);

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

    fn get_git_tree(author: &str, repo: &str) -> Result<serde_json::Value, String> {
        let client = Client::new();

        let sha = match Self::get_sha(&client, author, repo) {
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
            .header("User-Agent", "WinterOS")
            .send()
        {
            Ok(resp) => resp,
            Err(err) => return Err(err.to_string()),
        };

        response = Self::request_forbiden_fix(&client, response, &url);

        if response.status() != StatusCode::OK {
            return Err(format!("Failed to get git tree: {}", response.status()));
        }
        let json: serde_json::Value = response
            .text()
            .expect("Error during text convertion of request")
            .parse()
            .map_err(|e: serde_json::Error| e.to_string())
            .unwrap();

        use serde_json::Value::Object;
        if let Object(modules) = json {
            let res = match modules.get("tree") {
                Some(module) => Ok(module.clone()),
                None => Err(String::from("Unexpcted response format")),
            };
            return res;
        }
        Err(String::from("Unexpcted response format"))
    }

    pub fn new() -> Result<HardwareModule, String> {
        let all_module = match Self::get_git_tree("NixOS", "nixos-hardware") {
            Ok(ret) => ret,
            Err(err) => return Err(err),
        };

        let mut hardware = HardwareModule {
            module_common: vec![],
            module_computer: vec![],
        };
        hardware.module_common.reserve(20);
        hardware.module_computer.reserve(100);

        if let Some(list_all_module) = all_module.as_array() {
            for module in list_all_module {
                use serde_json::Value::Object;
                if let Object(module_content) = module {
                    let path: String =
                        match match module_content["path"].to_string().strip_prefix("\"") {
                            Some(string) => string,
                            None => continue,
                        }
                        .strip_suffix("\"")
                        {
                            Some(string) => string,
                            None => continue,
                        }
                        .trim()
                        .to_string();
                    if path.starts_with("common/") {
                        if path.ends_with("default.nix") {
                            hardware.module_common.push(path);
                        }
                    } else {
                        if path.ends_with("default.nix") {
                            hardware.module_computer.push(path);
                        }
                    }
                }
            }
        }
        Ok(hardware)
    }

    pub fn get_computer_module(&self) -> &Vec<String> {
        return &self.module_computer;
    }

    pub fn get_common_module(&self) -> &Vec<String> {
        return &self.module_common;
    }
}
