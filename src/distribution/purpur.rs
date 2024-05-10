use crate::distribution::{download_file, install_eula, install_server_jar, install_start};
use crate::error::*;
use futures_util::future::join3;
use inquire::Select;
use serde::Deserialize;
use spinners::{Spinner, Spinners};
use std::path::PathBuf;

pub struct Purpur {
    version: String,
}

impl Purpur {
    pub async fn new() -> Result<Self> {
        let mut sp = Spinner::new(Spinners::Dots, "Downloading metadata".into());
        let version_list = Self::get_versions().await?;
        sp.stop_and_persist("✔", "Finished downloading metadata".into());

        let mut options = version_list.versions;
        options.reverse();
        let version = Select::new("Select version", options).prompt()?;

        Ok(Self { version })
    }

    async fn get_versions() -> Result<VersionList> {
        let url = "https://api.purpurmc.org/v2/purpur/";
        let res = reqwest::get(url).await?;
        let body = res.text().await?;
        let ver = serde_json::from_str(&body)?;
        Ok(ver)
    }

    pub async fn install(&self, path: &PathBuf) -> Result<()> {
        let url = format!(
            "https://api.purpurmc.org/v2/purpur/{}/latest/download",
            self.version,
        );
        let content = download_file(&url).await?;

        let res = join3(
            install_server_jar(path, &content),
            install_eula(path),
            install_start(path),
        )
        .await;
        res.0?;
        res.1?;
        res.2?;

        Ok(())
    }
}

#[derive(Deserialize)]
struct VersionList {
    versions: Vec<String>,
}
