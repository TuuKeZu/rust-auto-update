use std::{process::Command, os::windows::process::CommandExt};

use rust_launcher::{update::*, utility::*};

type Error = Box<dyn std::error::Error>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    check_for_updates().await?;
    
    let version = get_version()?;

    println!("Running on {}", version.label);

    Ok(())
}
