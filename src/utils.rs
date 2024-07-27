use flate2::read::GzDecoder;
use std::{
    env::var_os,
    io::Read,
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

pub fn get_default_chatterino_path() -> Result<PathBuf, ()> {
    let machine_kind = if cfg!(linux) {
        Some("linux")
    } else if cfg!(windows) {
        Some("windows")
    } else {
        None
    };

    let path = match machine_kind {
        Some("windows") => {
            if let Some(mut roaming_path_buf) = var_os("APPDATA").map(PathBuf::from) {
                roaming_path_buf.push("Chatterino2");
                Ok(roaming_path_buf.to_owned())
            } else {
                Err(())
            }
        }
        Some("linux") => Ok(Path::new("~/.local/share/chatterino").to_owned()),
        _ => {
            println!("Unsupported OS, cannot locate Chatterino folder. Please use --path to specify the path.");
            Err(())
        }
    };

    path
}
