use regex::Regex;
use reqwest;
use serde_json;
use std::fs;
use std::{io::Read, path::Path};
use url::{Host::Domain, Url};

use crate::utils::{
    get_default_chatterino_path, get_files_from_gzip, handle_github_rate_limit, parse_plugins,
    print_plugins, write_plugin_data,
};
use crate::VERSION_STR;

pub fn get_plugin(
    plugin: &String,
    is_repo: bool,
    chatterino_path: Option<&String>,
) -> Result<(), String> {
    if !is_repo {
        return Err("Non repo plugins are not currently supported!".to_string());
    }

    // parse url
    let parsed_url = Url::parse(plugin).or(Err("Invalid URL".to_string()))?;

    // check if domain is github.com
    let domain = parsed_url.host().ok_or("Could not parse domain")?;
    if domain != Domain("github.com") {
        return Err("Invalid GitHub repository URL".to_string());
    }

    // extract owner and repo name from path
    let github_path_re = Regex::new(r"^/([^/]+)/([^/]+)/?$").unwrap();
    let captures = github_path_re
        .captures(parsed_url.path())
        .ok_or("Invalid GitHub repository URL")?;

    let owner = captures
        .get(1)
        .ok_or("Could not parse repository owner from URL")?
        .as_str();

    let repo = captures
        .get(2)
        .ok_or("Could not parse repository name from URL")?
        .as_str();

    // github api to get default branch
    let repo_info_url = format!("https://api.github.com/repos/{owner}/{repo}");
    let client = reqwest::blocking::Client::new();

    let request = client
        .get(repo_info_url)
        .header(
            "User-Agent",
            format!("Chatterino Plugin Manager {VERSION_STR}"),
        )
        .header("Accept", "application/json");

    let response = request.send().or(Err(
        "There was en error getting GitHub repository info".to_string()
    ))?;
    handle_github_rate_limit(&response)?;

    // parse response body as json and get `default_branch`
    let default_branch: String = if let Ok(json) = response.json::<serde_json::Value>() {
        serde_json::from_value(json.get("default_branch").unwrap().to_owned()).unwrap()
    } else {
        return Err("There was an error parsing the GitHub API response".to_string());
    };

    // get tarball
    let repo_tarball_url =
        format!("https://api.github.com/repos/{owner}/{repo}/tarball/{default_branch}");
    let client = reqwest::blocking::Client::new();

    let request = client.get(repo_tarball_url).header(
        "User-Agent",
        format!("Chatterino Plugin Manager {VERSION_STR}"),
    );

    let mut response = request.send().or(Err(
        "There was en error downloading GitHub repository tarball".to_string(),
    ))?;
    handle_github_rate_limit(&response)?;

    // write tarball to vec
    let mut buf: Vec<u8> = vec![];
    response
        .read_to_end(&mut buf)
        .or(Err("There was an writing the tarball".to_string()))?;

    let files = get_files_from_gzip(&buf);

    // get chatterino plugins folder path
    let chatterino_plugins_path = if let Some(chatterino_path) = chatterino_path {
        Path::new(chatterino_path).to_owned().join("Plugins")
    } else {
        get_default_chatterino_path()
            .or(Err("Chatterino path could no be automatically detected and no path was explicity specified".to_string()))?
            .join("Plugins")
    };

    // write to plugin folder
    write_plugin_data(chatterino_plugins_path, repo, files)?;

    Ok(())
}

pub fn list_plugins(chatterino_path: Option<&String>) -> Result<(), String> {
    // get chatterino plugins folder path
    let chatterino_plugins_path = if let Some(chatterino_path) = chatterino_path {
        Path::new(chatterino_path).to_owned().join("Plugins")
    } else {
        get_default_chatterino_path()
            .or(Err("Chatterino path could no be automatically detected and no path was explicity specified".to_string()))?
            .join("Plugins")
    };

    let plugins = parse_plugins(&chatterino_plugins_path)?;
    print_plugins(plugins);

    Ok(())
}

pub fn remove_plugin(chatterino_path: Option<&String>, plugin_name: String) -> Result<(), String> {
    // get chatterino plugins folder path
    let chatterino_plugins_path = if let Some(chatterino_path) = chatterino_path {
        Path::new(chatterino_path).to_owned().join("Plugins")
    } else {
        get_default_chatterino_path()
            .or(Err("Chatterino path could no be automatically detected and no path was explicity specified".to_string()))?
            .join("Plugins")
    };

    let plugins = parse_plugins(&chatterino_plugins_path)?;
    let plugin = plugins
        .iter()
        .find(|p| p.folder == plugin_name)
        .ok_or(format!("Plugin '{plugin_name}' not found."))?;

    fs::remove_dir_all(chatterino_plugins_path.join(&plugin.folder))
        .or(Err("There was an error removing the plugin"))?;

    println!("Removed {plugin_name}");

    Ok(())
}
