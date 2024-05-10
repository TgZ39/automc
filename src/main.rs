use crate::distribution::*;
use error::*;
use inquire::{Confirm, Select, Text};
use std::path::PathBuf;
use strum::IntoEnumIterator;

pub mod distribution;
mod error;

#[tokio::main]
async fn main() -> Result<()> {
    let directory = Text::new("Select directory").prompt()?;
    let dir = PathBuf::from(directory);

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
    };

    Text::new("Press <ENTER> to exit...").prompt()?;
    Ok(())
}
