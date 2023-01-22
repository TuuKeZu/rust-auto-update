use serde_derive::{Deserialize, Serialize};
use zip::write::FileOptions;
use std::fs::{self, File, OpenOptions};
use std::io::{Write, BufReader, BufWriter};
use std::path::Path;
use toml;
use std::env;

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum OsType {
    Windows,
    Linux,
    Unsupported
}

#[derive(Deserialize, Serialize)]
pub struct Data {
    pub version: Version
}

impl Data {
    pub fn as_toml(&self) -> String {
        toml::to_string(self).unwrap()
    }
}

#[derive(Deserialize, Serialize)]
pub struct Version {
    pub id: usize,
    pub label: String
}

impl Default for Version {
    fn default() -> Self {
        Self {
            id: 0,
            label: String::from("unset")
        }
    }
}

pub fn get_os() -> OsType {
    match env::consts::OS {
        "linux" => OsType::Linux,
        "windows" => OsType::Windows,
        _ => OsType::Unsupported
    }
}

pub fn get_version() -> Result<Version, std::io::Error> {
    let file_name = "Version.toml";

    file_exists(file_name, Data {version: Version::default() }.as_toml())?;

    let contents = fs::read_to_string(file_name)?;
    let data: Data = toml::from_str(&contents)?;

    Ok(data.version)
}

pub fn set_version(new_raw: String) -> Result<(), std::io::Error> {
    let file_name = "Version.toml";

    let file = OpenOptions::new().read(true).write(true).create(true).open(file_name)?;
    let mut writer = BufWriter::new(file);

    writer.write_all(new_raw.as_bytes())?;
    
    Ok(())
}


pub fn dir_exists(path: &str) -> Result<(), std::io::Error> {
    if Path::is_dir(Path::new(path)) {
        return Ok(());
    }
    fs::create_dir(path)?;
    Ok(())
}

pub fn file_exists(path: &str, default_raw: String) -> Result<(), std::io::Error> {
    if Path::is_file(Path::new(path)) {
        return Ok(());
    }

    let mut file = File::create(path)?;
    file.write_all(default_raw.as_bytes())?;
    Ok(())
}

