use pretty_duration::pretty_duration;
use regex::Regex;
use reqwest::{self, blocking::Response, header::HeaderValue};
use serde_json;
use std::{
    io::Read,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use url::{Host::Domain, Url};

use crate::utils::{get_default_chatterino_path, get_files_from_gzip};
use crate::VERSION_STR;

fn handle_github_rate_limit(response: &Response) -> Result<(), ()> {
    let status = response.status();
    if status.as_u16() == 403 || status.as_u16() == 429 {
        // rate limit reached
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
        let status_str = status.as_str();
        println!("GitHub API returned an unexpected status code: {status_str}");
        return Err(());
    }

    Ok(())
}

pub fn get_plugin(
    plugin: &String,
    is_repo: bool,
    chatterino_path: Option<&String>,
) -> Result<(), ()> {
    if !is_repo {
        println!("Non repo plugins are not currently supported!");
        return Err(());
    }

    // parse url
    let parsed_url = if let Ok(parsed) = Url::parse(plugin) {
        parsed
    } else {
        println!("Invalid URL");
        return Err(());
    };

    // check if domain is github.com
    if parsed_url.host() != Some(Domain("github.com")) {
        println!("Invalid GitHub repository URL");
        return Err(());
    }

    // extract owner and repo name from path
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
    // println!("{}", repo_info_url);
    let client = reqwest::blocking::Client::new();

    let request = client
        .get(repo_info_url)
        .header(
            "User-Agent",
            format!("Chatterino Plugin Manager {VERSION_STR}"),
        )
        .header("Accept", "application/json");

    let response = if let Ok(response) = request.send() {
        if handle_github_rate_limit(&response).is_ok() {
            response
        } else {
            return Err(());
        }
    } else {
        println!("There was en error getting GitHub repository info");
        return Err(());
    };

    // parse response body as json and get `default_branch`
    let default_branch: String = if let Ok(json) = response.json::<serde_json::Value>() {
        serde_json::from_value(json.get("default_branch").unwrap().to_owned()).unwrap()
    } else {
        println!("There was an error parsing the GitHub API response");
        return Err(());
    };

    println!("Default branch: {default_branch}");

    // get tarball
    // https://api.github.com/repos/<owner>/<repo>/tarball/<branch>

    let repo_tarball_url =
        format!("https://api.github.com/repos/{owner}/{repo}/tarball/{default_branch}");
    // println!("{}", repo_tarball_url);
    let client = reqwest::blocking::Client::new();

    let request = client.get(repo_tarball_url).header(
        "User-Agent",
        format!("Chatterino Plugin Manager {VERSION_STR}"),
    );

    let mut response = if let Ok(response) = request.send() {
        if handle_github_rate_limit(&response).is_ok() {
            response
        } else {
            return Err(());
        }
    } else {
        println!("There was en error getting GitHub repository info");
        return Err(());
    };

    let mut buf: Vec<u8> = vec![];
    if !response.read_to_end(&mut buf).is_ok() {
        println!("There was an error downloading the tarball");
        return Err(());
    }

    let files = get_files_from_gzip(&buf);

    // println!("{:?}", files);

    let chatterino_path = if let Some(chatterino_path) = chatterino_path {
        chatterino_path.to_owned()
    } else {
        if let Ok(path) = get_default_chatterino_path() {
            path.to_string_lossy().into_owned()
        } else {
            println!("Chatterino path could no be automatically detected and no path was explicity specified");
            return Err(());
        }
    };

    println!("{chatterino_path}");

    Ok(())
}
