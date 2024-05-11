use crate::distribution::{download_file, install_server_jar};
use crate::error::*;
use inquire::{Confirm, Select};
use serde::Deserialize;
use spinners::{Spinner, Spinners};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

pub struct Vanilla {
    download_url: String,
}

impl Vanilla {
    pub async fn new() -> Result<Self> {
        let mut sp = Spinner::new(Spinners::Dots, "Downloading metadata".into());
        let version_list = Self::get_versions().await?;
        sp.stop_and_persist("✔", "Finished downloading metadata".into());

        let options = {
            let mut out = version_list.versions;
            if Confirm::new("Only release versions?")
                .with_default(true)
                .prompt()?
            {
                out.retain(|v| v.channel == Channel::Release);
            }
            out
        };
        let version = Select::new("Select version", options).prompt()?;

        let mut sp = Spinner::new(Spinners::Dots, "Downloading version info".into());
        let version_info = reqwest::get(version.url).await?.text().await?;
        let url = gjson::get(&version_info, "downloads.server.url")
            .str()
            .to_owned();
        assert!(!url.is_empty());
        sp.stop_and_persist("✔", "Finished downloading version info".into());

        Ok(Self { download_url: url })
    }

    pub async fn install(&self, path: &PathBuf) -> Result<()> {
        let bytes = download_file(&self.download_url).await?;
        install_server_jar(path, &bytes).await?;

        Ok(())
    }

    async fn get_versions() -> Result<VersionList> {
        let url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
        let res = reqwest::get(url).await?;
        let body = res.text().await?;
        let ver = serde_json::from_str(&body)?;
        Ok(ver)
    }
}

#[derive(Deserialize)]
struct VersionList {
    versions: Vec<VersionInfo>,
}

#[derive(Deserialize)]
struct VersionInfo {
    #[serde(rename = "id")]
    version: String,
    #[serde(rename = "type")]
    channel: Channel,
    url: String,
}

impl Display for VersionInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.version)
    }
}

#[derive(Deserialize, Eq, PartialEq, Copy, Clone)]
enum Channel {
    #[serde(rename = "release")]
    Release,
    #[serde(rename = "snapshot", alias = "old_beta", alias = "old_alpha")]
    Snapshot,
}
