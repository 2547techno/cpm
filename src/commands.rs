use pretty_duration::pretty_duration;
use regex::Regex;
use reqwest::{self, header::HeaderValue};
use serde_json;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use url::{Host::Domain, Url};

use crate::VERSION_STR;

pub fn get_plugin(plugin: &String, is_repo: bool) -> Result<(), ()> {
    if !is_repo {
        println!("Non repo plugins are not currently supported!");
        return Ok(());
    }

    let parsed_url = if let Ok(parsed) = Url::parse(plugin) {
        parsed
    } else {
        println!("Invalid URL");
        return Err(());
    };

    if parsed_url.host() != Some(Domain("github.com")) {
        println!("Invalid GitHub repository URL");
        return Err(());
    }

    let github_path_re = Regex::new(r"^/([^/]+)/([^/]+)/?$").unwrap();
    let captures = if let Some(captures) = github_path_re.captures(parsed_url.path()) {
        captures
    } else {
        println!("Invalid GitHub repository URL");
        return Err(());
    };

    let owner = if let Some(owner) = captures.get(1) {
        owner.as_str()
    } else {
        println!("Could not parse repository owner from URL");
        return Err(());
    };

    let repo = if let Some(repo) = captures.get(2) {
        repo.as_str()
    } else {
        println!("Could not parse repository name from URL");
        return Err(());
    };

    // github api to get default branch
    // https://api.github.com/repos/<owner>/<repo>
    // "default_branch"
    let repo_info_url = format!("https://api.github.com/repos/{owner}/{repo}");
    println!("{}", repo_info_url);
    let client = reqwest::blocking::Client::new();

    let response = if let Ok(response) = client
        .get(repo_info_url)
        .header(
            "User-Agent",
            format!("Chatterino Plugin Manager {VERSION_STR}"),
        )
        .header("Accept", "application/json")
        .send()
    {
        let status = response.status();
        if status.as_u16() == 403 || status.as_u16() == 429 {
            // rate limit
            let default_header_value = HeaderValue::from_str("").unwrap();
            let reset_epoch = response
                .headers()
                .get("X-RateLimit-Reset")
                .unwrap_or(&default_header_value)
                .to_str()
                .unwrap_or("")
                .parse::<i32>()
                .unwrap_or(-1);
            let current_epoch = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i32;

            let mut error_str = "GitHub API rate limit reached!".to_owned();
            let duration_str = pretty_duration(
                &Duration::from_secs((reset_epoch - current_epoch) as u64),
                None,
            );
            if reset_epoch != -1 {
                error_str.push_str(&format!(" Resets in {}", duration_str));
            }

            println!("{}", error_str);
            return Err(());
        } else if !status.is_success() {
            // unexpected error
            println!("GitHub API returned an unexpected status code");
            return Err(());
        }

        response
    } else {
        println!("There was en error getting GitHub repository info");
        return Err(());
    };

    let json = if let Ok(json) = response.json::<serde_json::Value>() {
        json
    } else {
        println!("There was an error parsing the GitHub API response");
        return Err(());
    };

    // println!("{}", json);
    println!("200 OK");

    // get tarball
    // https://api.github.com/repos/<owner>/<repo>/tarball/<branch>
    Ok(())
}
