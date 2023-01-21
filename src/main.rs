use std::{process::Command, os::windows::process::CommandExt};

use rust_launcher::{update::*, utility::*};

type Error = Box<dyn std::error::Error>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    //get_version()?;
    //set_version(Data { version: Version { binary_id: 123, version_label: "beta 0.0.1".to_string()}}.as_json())?;
    check_for_updates().await?;
    download_binary("https://github.com/TuuKeZu/github-actions/releases/download/v0.0.3/github-actions_v0.0.3_x86_64-pc-windows-gnu.zip").await?;
    
    Ok(())
}
