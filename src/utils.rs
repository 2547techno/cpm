use flate2::read::GzDecoder;
use pretty_duration::pretty_duration;
use reqwest::{self, blocking::Response, header::HeaderValue};
use std::{
    env::var_os,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tar::Archive;

#[derive(Debug)]
pub struct ProjectPath {
    path_components: Vec<String>,
    is_dir: bool,
}

#[derive(Debug)]
pub struct ProjectFile {
    path: ProjectPath,
    content: Vec<u8>,
}

pub fn get_files_from_gzip(buf: &Vec<u8>) -> Vec<ProjectFile> {
    let dec = GzDecoder::new(&buf[..]);
    let mut archive = Archive::new(dec);
    let mut files = vec![];

    for file in archive.entries().unwrap() {
        let mut file = file.unwrap();

        let full_path = file.path().unwrap();
        let is_dir = file.header().entry_type().is_dir();

        let mut full_path_components = full_path
            .components()
            .map(|comp| comp.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        full_path_components.remove(0);

        if full_path_components.len() == 0 {
            continue;
        }

        let project_path = ProjectPath {
            is_dir: is_dir,
            path_components: full_path_components,
        };

        let mut file_content: Vec<u8> = Vec::new();
        file.read_to_end(&mut file_content).unwrap();

        files.push(ProjectFile {
            path: project_path,
            content: file_content,
        });
    }

    files
}

pub fn get_default_chatterino_path() -> Result<PathBuf, String> {
    let machine_kind = if cfg!(linux) {
        Some("linux")
    } else if cfg!(windows) {
        Some("windows")
    } else {
        None
    };

    match machine_kind {
        Some("windows") => {
            let mut roaming_path_buf = var_os("APPDATA").map(PathBuf::from).ok_or(
                "Could not read %APPDATA% environment variable. Please use --path instead.",
            )?;
            roaming_path_buf.push("Chatterino2");
            Ok(roaming_path_buf.to_owned())
        }
        Some("linux") => Ok(Path::new("~/.local/share/chatterino").to_owned()),
        _ => Err(
            "Unsupported OS, cannot locate Chatterino folder. Please use --path instead."
                .to_string(),
        ),
    }
}

pub fn handle_github_rate_limit(response: &Response) -> Result<(), String> {
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

        return Err(error_str);
    } else if !status.is_success() {
        let status_str = status.as_str();
        return Err(format!(
            "GitHub API returned an unexpected status code: {status_str}"
        ));
    }

    Ok(())
}

pub fn write_plugin_data(
    base_path: PathBuf,
    name: &str,
    files: Vec<ProjectFile>,
) -> Result<(), String> {
    if !base_path.is_dir() {
        return Err("Plugins folder not found in Chatterino folder".to_string());
    }

    let plugin_path = base_path.join(name);

    // check if a plugin with the same name is already installed
    if plugin_path.is_dir() {
        return Err(format!(
            "A plugin with the name {name} is already installed."
        ));
    }

    if fs::create_dir_all(&plugin_path).is_err() {
        return Err(format!("There was an error creating {name}"));
    }
    println!("Wrote {}", &plugin_path.to_string_lossy());

    for file in files {
        let subpath = file.path.path_components.join("/");
        let path = plugin_path.join(subpath);

        if file.path.is_dir {
            if fs::create_dir_all(&path).is_err() {
                return Err(format!(
                    "There was an error creating {}",
                    file.path.path_components.join("/")
                ));
            }
        } else {
            let f = File::create_new(&path);

            if let Ok(mut f) = f {
                if f.write_all(&file.content).is_err() {
                    return Err(format!(
                        "There was an writing to {}",
                        file.path.path_components.join("/")
                    ));
                }
            } else {
                return Err(format!(
                    "There was an error creating {}",
                    file.path.path_components.join("/")
                ));
            }
        }
        println!("Wrote {}", path.to_string_lossy());
    }

    Ok(())
}
