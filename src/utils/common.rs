use std::{env::var_os, io::Read, path::PathBuf};

use flate2::read::GzDecoder;
use serde_json::{Map, Value};
use tar::Archive;

#[derive(Debug)]
pub struct ProjectPath {
    pub path_components: Vec<String>,
    pub is_dir: bool,
}

#[derive(Debug)]
pub struct ProjectFile {
    pub path: ProjectPath,
    pub content: Vec<u8>,
}
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PluginPermission {
    pub type_: String,
}

impl PluginPermission {
    pub fn from_map(map: &Map<String, Value>) -> Result<PluginPermission, ()> {
        let type_val: String = map
            .get("type")
            .map(|v| serde_json::from_value(v.clone()).or(Err(())))
            .unwrap_or(Err(()))?;

        Ok(PluginPermission { type_: type_val })
    }
}

#[derive(Debug, Clone)]
pub struct Plugin {
    pub folder: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub authors: Vec<String>,
    pub tags: Vec<String>,
    pub version: Option<String>,
    pub licence: Option<String>,
    pub permissions: Vec<PluginPermission>,
}

impl Plugin {
    pub fn new() -> Self {
        Plugin {
            folder: String::new(),
            name: None,
            description: None,
            homepage: None,
            authors: Vec::new(),
            tags: Vec::new(),
            version: None,
            licence: None,
            permissions: Vec::new(),
        }
    }
}

/// Extract files from .tar.gz file
///
/// ### Arguments
/// * `buf` a .tar.gz file in vec of bytes
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

/// Get the default Chatterino path based on OS
pub fn get_default_chatterino_path() -> Result<PathBuf, String> {
    let machine_kind = if cfg!(target_os = "linux") {
        Some("linux")
    } else if cfg!(target_os = "windows") {
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
        Some("linux") => {
            let mut home_path_buf = var_os("HOME")
                .map(PathBuf::from)
                .ok_or("Could not read $HOME environment variable. Please use --path instead.")?;
            home_path_buf.push(".local/share/chatterino");
            Ok(home_path_buf.to_owned())
        }
        _ => Err(
            "Unsupported OS, cannot locate Chatterino folder. Please use --path instead."
                .to_string(),
        ),
    }
}
