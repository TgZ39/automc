use crate::args::Args;
use crate::config::Config;
use crate::distribution::*;
use crate::java::java_versions;
use clap::Parser;
use error::*;
use futures_util::future::join;
use inquire::{Confirm, Select, Text};
use std::path::PathBuf;
use strum::IntoEnumIterator;

mod args;
mod config;
mod distribution;
mod error;
mod java;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut config = Config::load()?;

    let dir: PathBuf = Text::new("Select directory")
        .with_help_message("leave empty for current directory")
        .prompt()?
        .into();

    let mut options: Vec<Distribution> = Distribution::iter().collect();
    options.sort_by_key(|a| a.to_string());

    let distribution = Select::new("Select distribution", options).prompt()?;

    if !config.accepted_eula {
        if !Confirm::new("Do you accept the EULA?")
            .with_help_message("https://www.minecraft.net/eula")
            .with_default(true)
            .prompt()?
        {
            return Ok(());
        } else {
            config.accepted_eula = true;
            config.save()?;
        }
    }

    let java_path = match args.java_path {
        Some(path) => PathBuf::from(path),
        None => {
            let mut java_versions = java_versions()?
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<String>>();
            java_versions.push("Custom".to_string());
            java_versions.push("use JAVA_HOME".to_string());

            let selected = Select::new("Select java version", java_versions).prompt()?;

            let path = match selected.as_str() {
                "Custom" => Text::new("Path to custom java binary")
                    .with_help_message("eg. /usr/lib/jvm/bin/java")
                    .prompt()?,
                "use JAVA_HOME" => "java".to_string(),
                _ => selected,
            };

            PathBuf::from(path)
        }
    };

    let start_script = install_start_script(&dir, &java_path);
    let eula = install_eula(&dir);

    let res = join(start_script, eula).await;
    res.0?;
    res.1?;

    match distribution {
        Distribution::Paper => Paper::new().await?.install(&dir).await?,
        Distribution::Folia => Folia::new().await?.install(&dir).await?,
        Distribution::Velocity => Velocity::new().await?.install(&dir).await?,
        Distribution::Purpur => Purpur::new().await?.install(&dir).await?,
        Distribution::Fabric => Fabric::new().await?.install(&dir).await?,
        Distribution::Vanilla => Vanilla::new().await?.install(&dir).await?,
        Distribution::Spigot => Spigot::new().await?.install(&dir, &java_path).await?,
    };

    Text::new("Press <ENTER> to exit...").prompt()?;
    Ok(())
}
