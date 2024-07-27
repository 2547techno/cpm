use flate2::read::GzDecoder;
use std::{
    env::var_os,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
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

pub fn write_plugin_data(
    base_path: PathBuf,
    name: &str,
    files: Vec<ProjectFile>,
) -> Result<(), String> {
    if !base_path.is_dir() {
        return Err("Plugins folder not found in Chatterino folder".to_string());
    }

    //TODO: Check if plugin is already installed - https://trello.com/c/Bc1p8cWq/4-check-if-plugin-is-already-installed

    let plugin_path = base_path.join(name);

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
