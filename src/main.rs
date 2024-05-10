use crate::distribution::*;
use error::*;
use inquire::validator::Validation;
use inquire::{Select, Text};
use std::path::PathBuf;
use std::str::FromStr;
use strum::IntoEnumIterator;

pub mod distribution;
mod error;

#[tokio::main]
async fn main() -> Result<()> {
    let validator = |path: &str| {
        if let Ok(path) = PathBuf::from_str(path) {
            if path.is_dir() {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid("Path must be a directory".into()))
            }
        } else {
            Ok(Validation::Invalid("Input must be a path".into()))
        }
    };
    let directory = Text::new("Select directory")
        .with_validator(validator)
        .prompt()?;
    let dir = PathBuf::from_str(&directory)?;

    let options = Distribution::iter().collect();
    let distribution = Select::new("Select distribution", options).prompt()?;

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
