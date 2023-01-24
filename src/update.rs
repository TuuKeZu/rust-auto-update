use anyhow::Result;
use colored::*;
use reqwest::header::USER_AGENT;
use std::{
    collections::HashMap,
    env,
    fmt::{self, format},
    fs::{self, File},
    io::Cursor, path::Path,
};
use zip::ZipArchive;

use crate::utility::*;

const BINARY_NAME: &str = env!("CARGO_PKG_NAME");

/// Struct used to construct fields for `check_for_updates()` method.
///
/// # Examples
/// ```ignore
/// //Create a update builder that retrieves latest update from https://github.com/TuuKeZu/rust-auto-update/releases
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     UpdateBuilder::new()
///         .set_verbose(true)
///         .set_github_user("TuuKeZu")
///         .set_github_repo("rust-auto-update")
///         .set_binary_path(OsType::Windows, "x86_64-pc-windows-gnu.zip")
///         .set_binary_path(OsType::Linux, "x86_64-apple-darwin.zip")
///         .check_for_updates()
///         .await?;
///
///     Ok(())
/// }
/// ```

#[derive(Debug, Default)]
pub struct UpdateBuilder {
    verbose: bool,
    github_user: Option<String>,
    github_repo: Option<String>,
    binary_map: HashMap<OsType, String>,
}

impl UpdateBuilder {
    /// Creates a new `UpdateBuilder`
    pub fn new() -> Self {
        Self {
            verbose: true,
            github_user: None,
            github_repo: None,
            binary_map: HashMap::new(),
        }
    }

    /// Set to `true` by default. Controls weather to show output or not
    pub fn set_verbose(mut self, value: bool) -> Self {
        self.verbose = value;
        self
    }

    /// Set the GitHub `username`.
    pub fn set_github_user(mut self, username: &str) -> Self {
        self.github_user = Some(username.to_string());
        self
    }

    /// Set the GitHub repository. The repository must be public and the specified `username` must own the `repository`
    pub fn set_github_repo(mut self, name: &str) -> Self {
        self.github_repo = Some(name.to_string());
        self
    }

    /// Set the path for operating specific binary.
    /// **Note that by not providing any path your application won't support any operating system**
    pub fn set_binary_path(mut self, os: OsType, path: &str) -> Self {
        self.binary_map.insert(os, path.to_string());
        self
    }

    /// Check and install the latest update for the current operating system.
    pub async fn check_for_updates(mut self) -> Result<()> {
        assert!(self.github_user.is_some(), "GitHub user must be specified");
        assert!(self.github_repo.is_some(), "GitHub repo must be specified");

        println!("{}", "Checking for updates:".dimmed());
        let version = get_version()?;

        // Check for network connection
        if online::check(None).is_err() {
            return Err(
                UpdateError("Update failed: Missing Network connection".to_string()).into(),
            );
        }

        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            &self.github_user.as_ref().unwrap(),
            &self.github_repo.as_ref().unwrap()
        );
        let client = reqwest::Client::new();

        let resp = client
            .get(url.clone())
            .header(USER_AGENT, "rust-auto-update")
            .send()
            .await?;

        if resp.status() != 200 {
            return Err(UpdateError(format!("Failed to retrieve latest release <{url}>")).into());
        }

        let json: serde_json::Value = serde_json::from_str(&resp.text().await?)?;
        let id = &json["id"].as_i64().unwrap();
        let version_label = &json["tag_name"].as_str().unwrap();

        if version.id as i64 == *id {
            println!("{} {}", "> Up to date".dimmed(), version_label.green());
            return Ok(());
        }

        if version.id == 0 {
            println!(
                "{} {}",
                "> Running initial setup".dimmed(),
                version_label.green()
            );
        } else {
            println!(
                "{} {} {} {}",
                "> Update available:".dimmed(),
                version.label.yellow(),
                "=>".dimmed(),
                version_label.green()
            );
        }

        let os = get_os();
        let os_path = self.binary_map.get(&os);

        if os_path.is_none() {
            return Err(
                UpdateError("No binary was found for this operating system".to_string()).into(),
            );
        }

        let url = format!(
            "https://github.com/{}/{}/releases/download/{}/{}_{}_{}",
            self.github_user.as_ref().unwrap(),
            self.github_repo.as_ref().unwrap(),
            version_label,
            self.github_repo.as_ref().unwrap(),
            version_label,
            os_path.unwrap()
        );

        self.download_binary(os, &url).await?;
        set_version(
            Data {
                version: Version {
                    id: *id as usize,
                    label: version_label.to_string(),
                },
            }
            .as_toml(),
        )?;

        Ok(())
    }

    async fn download_binary(&mut self, os: OsType, url: &str) -> Result<()> {
        println!("{} {}", "> Installing".dimmed(), url.italic().white());
        dir_exists("version-tmp")?;
        dir_exists("version-cache")?;

        let tmp_exec_name = format!("version-tmp/{}.zip", self.github_repo.as_ref().unwrap());
        let exec_name = format!("tmp-{}", self.github_repo.as_ref().unwrap());

        println!("{}", "> Downloading latest binary...".dimmed());
        let response = reqwest::get(url).await?;

        if response.status() != 200 {
            return Err(
                UpdateError(format!("Failed to access installation binary <{url}>")).into(),
            );
        }

        let mut file = std::fs::File::create(tmp_exec_name.clone())?;

        let mut content = Cursor::new(response.bytes().await?);
        std::io::copy(&mut content, &mut file)?;

        println!("{}", "> Unzipping archived packages...".dimmed());
        let archive = File::open(tmp_exec_name)?;
        let mut archive = ZipArchive::new(archive)?;

        
        let executable = env::current_exe()?;
        let exec_path = executable.as_path();
        let exec_dir = exec_path.parent().unwrap();
        
        let mut executable = archive.by_index(0)?;

        let mut file = match os {
            OsType::Windows => {
                // On windows the executable needs to have a file type
                let exec_path = exec_dir.join(format!("{}.exe", exec_name));
                dbg!(&exec_path);
                std::fs::File::create(exec_path)?
            }
            OsType::Linux | OsType::MacOs => {
                let exec_path = exec_dir.join(format!("{}", exec_name));

                let file = std::fs::File::create(exec_path.clone())?;
                // On Unix based systems the file must be set to an executable
                cfg_if::cfg_if! {
                    if #[cfg(unix)] {
                        use std::os::unix::fs::PermissionsExt;
                        file.set_permissions(fs::Permissions::from_mode(0o777))?;
                    }
                }
                file
            }
            OsType::Unsupported => {
                return Err(UpdateError("Unsupported operating system".to_string()).into());
            }
        };

        std::io::copy(&mut executable, &mut file)?;

        println!("{}", "> Finalizing... ".dimmed());

        // MOVE (current) executable => cache
        // Remove .exe extentsion from the file to prohibit curious Windows users accidentally clicking cached executables.
        fs::rename(exec_path, "version-cache/last")?;
        
        // RENAME tmp-executable => executable
        match os {
            OsType::Windows => fs::rename(exec_dir.join(format!("{exec_name}.exe")), exec_path)?,
            OsType::Linux | OsType::MacOs => fs::rename(exec_dir.join(exec_name), exec_path)?,
            OsType::Unsupported => {
                return Err(UpdateError("Unsupported operating system".to_string()).into());
            }
        }

        println!("{} {}", ">".dimmed(), "Done!".green());
        Ok(())
    }
}

#[derive(Debug)]
pub struct UpdateError(String);

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Update failed: {}", self.0)
    }
}

impl std::error::Error for UpdateError {}
