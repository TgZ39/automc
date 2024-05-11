use crate::args::Args;
use crate::distribution::*;
use crate::java::installed_versions;
use clap::Parser;
use error::*;
use inquire::{Confirm, Select, Text};
use std::path::PathBuf;
use strum::IntoEnumIterator;

mod args;
mod distribution;
mod error;
mod java;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let dir = Text::new("Select directory").prompt()?;
    let dir = PathBuf::from(dir);

    let options = Distribution::iter().collect();
    let distribution = Select::new("Select distribution", options).prompt()?;

    if !Confirm::new("Do you accept the EULA? (https://www.minecraft.net/eula)")
        .with_default(true)
        .prompt()?
    {
        return Ok(());
    }

    match distribution {
        Distribution::Paper => Paper::new().await?.install(&dir).await?,
        Distribution::Folia => Folia::new().await?.install(&dir).await?,
        Distribution::Velocity => Velocity::new().await?.install(&dir).await?,
        Distribution::Purpur => Purpur::new().await?.install(&dir).await?,
        Distribution::Fabric => Fabric::new().await?.install(&dir).await?,
        Distribution::Vanilla => Vanilla::new().await?.install(&dir).await?,
    };
    install_eula(&dir).await?;

    let java_path = if let Some(path) = args.java_path {
        PathBuf::from(&path)
    } else {
        let options = installed_versions()?;
        let java_version = Select::new("Select Java version", options).prompt()?;
        PathBuf::from(&java_version)
    };
    install_start_script(&dir, &java_path).await?;

    Text::new("Press <ENTER> to exit...").prompt()?;
    Ok(())
}
