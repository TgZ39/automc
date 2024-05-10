use error::*;
use inquire::{Select, Text};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use strum::IntoEnumIterator;
use tokio::fs;
use crate::distribution::*;

mod error;
pub mod distribution;

#[tokio::main]
async fn main() -> Result<()> {
    let directory = Text::new("Select directory").prompt()?;
    let directory = PathBuf::from_str(&directory)?;

    let options = Distribution::iter().collect();
    let distribution = Select::new("Select distribution", options).prompt()?;

    let content = match distribution {
        Distribution::Paper => {
            Paper::new().await?.download().await?
        }
        Distribution::Folia => {
            Folia::new().await?.download().await?
        }
        Distribution::Velocity => {
            Velocity::new().await?.download().await?
        }
        Distribution::Purpur => {
            Purpur::new().await?.download().await?
        }
    };

    fs::create_dir_all(&directory).await?;

    let mut path = directory.clone();
    path.push("server.jar");
    let mut server_jar = File::create(path)?;
    server_jar.write_all(&content)?;

    let mut path = directory.clone();
    path.push("eula.txt");
    let mut eula = File::create(path)?;
    eula.write_all(b"eula=true")?;

    Text::new("Press <ENTER> to exit...").prompt()?;
    Ok(())
}
