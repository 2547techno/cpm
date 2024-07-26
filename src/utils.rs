use flate2::read::GzDecoder;
// use std::{env::var_os, path::PathBuf};
use tar::Archive;

pub fn decompress_gzip_tar(buf: &Vec<u8>) {
    let dec = GzDecoder::new(&buf[..]);
    let mut archive = Archive::new(dec);

    for file in archive.entries().unwrap() {
        let mut file = file.unwrap();

        let full_path = file.path().unwrap();
        let is_dir = file.header().entry_type().is_dir();

        let mut full_path_components = full_path.iter();

        let component = full_path_components.next();
        if component.is_none() {
            continue;
        }

        // let path_components

        let path_components = full_path_components.collect::<Vec<_>>();
        if path_components.len() == 0 {
            continue;
        }

        println!("{:?}", full_path);
        println!("{is_dir} {:?}\n", path_components);
    }

    // windows
    // println!("{:?}", var_os("APPDATA").map(PathBuf::from));
    // $APPDATA/Chatterino2

    // linux
    // ~/.local/share/chatterino
}
