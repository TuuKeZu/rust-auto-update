
use std::{io::{Cursor}, fs::{File, self}, env};
use reqwest::header::USER_AGENT;
use zip::ZipArchive;

use crate::{utility::*, process::get_os};

const BINARY_NAME: &str = env!("CARGO_PKG_NAME");
type Error = Box<dyn std::error::Error>;

pub async fn check_for_updates() -> Result<(), Error> {
    let version = get_version()?;
    let os = get_os();

    let url = "https://api.github.com/repos/TuuKeZu/github-actions/releases/latest";
    let client = reqwest::Client::new();

    let resp = client
        .get(url)
        .header(USER_AGENT, "rust-auto-update")
        .send()
        .await?;

    let json: serde_json::Value = serde_json::from_str(&resp.text().await?)?;
    let id = &json["id"].as_i64().unwrap();
    let version_label = &json["name"].as_str().unwrap();
    
    if version.binary_id as i64 == *id {
        println!("up to date");
        return Ok(())
    }

    set_version( Data { version: Version { binary_id: *id as usize, version_label: version_label.to_string() }}.as_toml())?;

    println!("{:#?} {:#?}", json, id);

    Ok(())
}

pub async fn download_binary(url: &str) -> Result<(), Error> {
    dir_exists("version-tmp")?;
    dir_exists("version-cache")?;

    let tmp_exec_name = format!("version-tmp/{BINARY_NAME}.zip");
    let exec_name = format!("tmp-{BINARY_NAME}.exe");
    
    println!("> downloading latest binary...");
    let response = reqwest::get(url).await?;
    let mut file = std::fs::File::create(tmp_exec_name.clone())?;
    let mut content =  Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;

    println!("> unzipping archived packages...");
    let archive = File::open(tmp_exec_name)?;
    let mut archive = ZipArchive::new(archive)?;
    
    let mut executable = archive.by_index(0)?;
    let mut file = std::fs::File::create(exec_name.clone())?;

    std::io::copy(&mut executable, &mut file)?;

    println!("> finalizing... ");
    let executable = env::current_exe()?;
    let exec_path = executable.as_path();
    let exec_dir = exec_path.parent().unwrap();

    // MOVE tmp-executable => binary
    fs::rename(exec_name.clone(), exec_dir.join(exec_name.clone()))?;

    // MOVE (current) executable => cache
    fs::rename(exec_path, format!("version-cache/last.exe"))?;

    // RENAME tmp-executable => executable
    fs::rename(exec_dir.join(exec_name), exec_path)?;

    println!("> done!");
    Ok(())
}