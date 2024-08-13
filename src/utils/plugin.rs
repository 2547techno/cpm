use serde_json::Value;
use tabled::{builder::Builder, settings::Style};

use super::common::{Plugin, PluginPermission, ProjectFile};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
};

/// Write plugin files to Plugins/ folder
///
/// ### Arguments
/// * `base_path` path to Chatterino folder
/// * `name` name of plugin to install (will be the folder name in `Plugins/`)
/// * `files` plugin files
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

/// Get plugin metadata from folder
///
/// ### Arguments
/// * `plugin_path` path to plugin folder
/// * `folder_name` plugin folder name
pub fn parse_plugin(plugin_path: PathBuf, folder_name: String) -> Result<Option<Plugin>, String> {
    let mut plugin = Plugin::new();

    let info_file_path = plugin_path.join("info.json");
    if !info_file_path.is_file() {
        return Ok(None);
    }

    let mut info_file = File::open(info_file_path)
        .or(Err("There was an error reading the info.json plugin file"))?;

    let mut info_file_buf = Vec::new();
    info_file
        .read_to_end(&mut info_file_buf)
        .or(Err("There was an error reading the info.json plugin file"))?;

    let buf: String = String::from_utf8(info_file_buf)
        .or(Err("There was an error decoding the info.json plugin file"))?;

    let json: Value = serde_json::from_str(buf.as_str())
        .or(Err("There was an error parsing the info.json plugin file"))?;

    plugin.folder = folder_name;
    plugin.name = json
        .get("name")
        .map(|v| serde_json::from_value(v.clone()).unwrap());
    plugin.description = json
        .get("description")
        .map(|v| serde_json::from_value(v.clone()).unwrap());
    plugin.homepage = json
        .get("homepage")
        .map(|v| serde_json::from_value(v.clone()).unwrap());
    plugin.version = json
        .get("version")
        .map(|v| serde_json::from_value(v.clone()).unwrap());
    plugin.licence = json
        .get("licence")
        .map(|v| serde_json::from_value(v.clone()).unwrap());

    let authors: Vec<Value> = Vec::new();
    let authors: Vec<String> = json
        .get("authors")
        .map(|v| v.as_array())
        .unwrap_or(None)
        .unwrap_or(&authors)
        .to_owned()
        .iter()
        .map(|v| serde_json::from_value(v.clone()).unwrap())
        .collect();
    plugin.authors = authors;

    let tags: Vec<Value> = Vec::new();
    let tags: Vec<String> = json
        .get("tags")
        .map(|v| v.as_array())
        .unwrap_or(None)
        .unwrap_or(&tags)
        .to_owned()
        .iter()
        .map(|v| serde_json::from_value(v.clone()).unwrap())
        .collect();
    plugin.tags = tags;

    let permissions: Vec<Value> = Vec::new();
    let permissions: Vec<PluginPermission> = json
        .get("permissions")
        .map(|v| v.as_array())
        .unwrap_or(None)
        .unwrap_or(&permissions)
        .to_owned()
        .iter()
        .map(|v| PluginPermission::from_map(v.as_object().unwrap()).unwrap())
        .collect();
    plugin.permissions = permissions;

    Ok(Some(plugin))
}

/// Get all plugins metadata
///
/// ### Arguments
/// * `path` path to `Plugins/` folder
pub fn parse_plugins(path: &PathBuf) -> Result<Vec<Plugin>, String> {
    let entries = fs::read_dir(path).or(Err("Could not read Plugins/ folder"))?;
    let mut plugins: Vec<Plugin> = Vec::new();

    let file_read_err_str = "Could not read file in Plugins/ folder";
    for entry in entries {
        let dir_entry = entry.or(Err(file_read_err_str))?;
        let file_type = dir_entry.file_type().or(Err(file_read_err_str))?;
        let file_name = dir_entry.file_name().to_string_lossy().to_string();

        if !file_type.is_dir() {
            continue;
        }

        let plugin_path = dir_entry.path();

        let plugin = parse_plugin(plugin_path, file_name)?;
        match plugin {
            Some(plugin) => plugins.push(plugin),
            None => continue,
        };
    }

    Ok(plugins)
}

pub fn print_plugins(plugins: Vec<Plugin>) {
    let mut builder = Builder::default();
    builder.push_record(["Installation Name", "Plugin Name", "Version"]);

    for plugin in plugins {
        builder.push_record([
            plugin.folder,
            format!("({})", plugin.name.unwrap_or("Unknown".to_string())),
            format!("v{}", plugin.version.unwrap_or("Unknown".to_string())),
        ]);
    }

    let table = builder.build().with(Style::rounded()).to_string();
    println!("{table}");
}

pub fn print_plugin_info(plugin: Plugin) {
    let mut builder = Builder::new();
    builder.push_record(["Folder".to_string(), plugin.folder]);
    builder.push_record([
        "Name".to_string(),
        plugin.name.unwrap_or("Unknown".to_string()),
    ]);
    builder.push_record([
        "Description".to_string(),
        plugin.description.unwrap_or("None".to_string()),
    ]);
    builder.push_record([
        "Homepage".to_string(),
        plugin.homepage.unwrap_or("None".to_string()),
    ]);
    builder.push_record(["Authors".to_string(), plugin.authors.join(", ")]);
    builder.push_record(["Tags".to_string(), plugin.tags.join(", ")]);
    builder.push_record([
        "Version".to_string(),
        plugin.version.unwrap_or("Unknown".to_string()),
    ]);
    builder.push_record([
        "Licence".to_string(),
        plugin.licence.unwrap_or("None".to_string()),
    ]);
    builder.push_record([
        "Permissions".to_string(),
        plugin
            .permissions
            .iter()
            .map(|p| p.type_.clone())
            .collect::<Vec<String>>()
            .join(", "),
    ]);

    let table = builder.build().with(Style::ascii_rounded()).to_string();
    println!("{table}");
}
