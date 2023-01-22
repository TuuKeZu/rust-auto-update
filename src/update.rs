
use std::{io, io::Cursor, fs::{File, self}, env, fmt::{format, self}};
use reqwest::header::USER_AGENT;
use zip::ZipArchive;
use anyhow::Result;

use crate::{utility::*, process::{get_os, ProcessType}};

const BINARY_NAME: &str = env!("CARGO_PKG_NAME");

const GITHUB_REPO_API: &str = "https://api.github.com/repos/TuuKeZu/github-actions";
const GITHUB_REPO: &str = "https://github.com/TuuKeZu/github-actions";

const MAC_BINARY: &str = "x86_64-unknown-linux-musl.zip";
const WINDOWS_BINARY: &str = "x86_64-pc-windows-gnu.zip";
const LINUX_BINARY: &str = "x86_64-apple-darwin.zip";


#[derive(Debug)]
pub struct UpdateError(String);

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Update failed: {}", self.0)
    }
}

impl std::error::Error for UpdateError {}

pub async fn check_for_updates() -> Result<()> {
    println!("Checking for updates:");
    let version = get_version()?;
    
    // Check for network connection
    if online::check(None).is_err() {
        let mut a = String::new();
        // Panic if no version has been installed
        if version.id == 0 {
            return Err( UpdateError("Failed to initilize binary: Network error".to_string()).into() );
        }

        println!("You are not connected to any network.");
        println!("Do you wish to continue offline? [yes / no]");
        io::stdin().read_line(&mut a).unwrap();
        
        if a.trim() == "yes" || a.trim() == "y" {
            return Ok(());
        }

        return Err( UpdateError("Update aborted".to_string()).into() );
    }
    
    let url = format!("{GITHUB_REPO_API}/releases/latest");
    let client = reqwest::Client::new();

    let resp = client
        .get(url)
        .header(USER_AGENT, "rust-auto-update")
        .send()
        .await?;

    let json: serde_json::Value = serde_json::from_str(&resp.text().await?)?;
    let id = &json["id"].as_i64().unwrap();
    let version_label = &json["tag_name"].as_str().unwrap();
    
    if version.id as i64 == *id {
        println!("> up to date");
        return Ok(())
    }

    if version.id == 0 {
        println!("> Installing {}", version_label);
    } else {
        println!("> Update available: {} => {}", version.label, version_label);
    }

    let os = get_os();
    let url = match os {
        ProcessType::Windows => format!("{GITHUB_REPO}/releases/download/{version_label}/{BINARY_NAME}_{version_label}_{WINDOWS_BINARY}"),
        ProcessType::Linux => format!("{GITHUB_REPO}/releases/download/{version_label}/{BINARY_NAME}_{version_label}_{LINUX_BINARY}"),
        ProcessType::Unsupported => panic!("Unsupported operating system"),
    };

    download_binary(&url).await?;
    set_version( Data { version: Version { id: *id as usize, label: version_label.to_string() }}.as_toml())?;

    Ok(())
}

pub async fn download_binary(url: &str) -> Result<()> {
    println!("{}", url);
    dir_exists("version-tmp")?;
    dir_exists("version-cache")?;

    let tmp_exec_name = format!("version-tmp/{BINARY_NAME}.zip");
    let exec_name = format!("tmp-{BINARY_NAME}.exe");
    
    println!("> downloading latest binary...");
    let response = reqwest::get(url).await?;

    if response.status() != 200 {
        return Err( UpdateError(format!("Failed to access installation binary <{url}>")).into() )
    }

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