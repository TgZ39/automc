use crate::args::Args;
use crate::distribution::*;
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
        Distribution::Paper => Paper::new().await?.install(&dir, args).await?,
        Distribution::Folia => Folia::new().await?.install(&dir, args).await?,
        Distribution::Velocity => Velocity::new().await?.install(&dir, args).await?,
        Distribution::Purpur => Purpur::new().await?.install(&dir, args).await?,
        Distribution::Fabric => Fabric::new().await?.install(&dir, args).await?,
        Distribution::Vanilla => Vanilla::new().await?.install(&dir, args).await?,
        Distribution::Spigot => Spigot::new().await?.install(&dir, args).await?,
    };

    Text::new("Press <ENTER> to exit...").prompt()?;
    Ok(())
}
