use serde_derive::{Deserialize, Serialize};
use zip::write::FileOptions;
use std::fs::{self, File, OpenOptions};
use std::io::{Write, BufReader, BufWriter};
use std::path::Path;
use toml;

type Error = Box<dyn std::error::Error>;

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
    pub binary_id: usize,
    pub version_label: String
}

impl Default for Version {
    fn default() -> Self {
        Self {
            binary_id: 0,
            version_label: String::from("unset")
        }
    }
}

pub fn get_version() -> Result<Version, Error> {
    let file_name = "Version.toml";

    file_exists(file_name, Data {version: Version::default() }.as_toml())?;

    let contents = fs::read_to_string(file_name)?;

    let data: Data = toml::from_str(&contents)?;

    Ok(data.version)
}

pub fn set_version(new_raw: String) -> Result<(), Error> {
    let file_name = "Version.toml";

    let file = OpenOptions::new().read(true).write(true).create(true).open(file_name)?;
    let mut writer = BufWriter::new(file);

    writer.write_all(new_raw.as_bytes())?;
    
    Ok(())
}


pub fn dir_exists(path: &str) -> Result<(), Error> {
    if Path::is_dir(Path::new(path)) {
        return Ok(());
    }
    fs::create_dir(path)?;
    Ok(())
}

pub fn file_exists(path: &str, default_raw: String) -> Result<(), Error> {
    if Path::is_file(Path::new(path)) {
        return Ok(());
    }

    let mut file = File::create(path)?;
    file.write_all(default_raw.as_bytes())?;
    Ok(())
}

