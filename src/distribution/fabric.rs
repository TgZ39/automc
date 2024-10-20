use crate::distribution::{download_file, install_server_jar};
use crate::error::*;
use futures_util::future::join3;
use inquire::{Confirm, Select};
use serde::Deserialize;
use spinners::{Spinner, Spinners};
use std::fmt::{Display, Formatter};
use std::path::Path;

pub struct Fabric {
    version: String,
    loader: String,
    installer: String,
}

impl Fabric {
    pub async fn new() -> Result<Self> {
        let mut sp = Spinner::new(Spinners::Dots, "Downloading metadata".into());
        let lists = join3(
            Self::get_versions(),
            Self::get_installers(),
            Self::get_loaders(),
        )
        .await;
        let (version_list, installer_list, loader_list) = (lists.0?, lists.1?, lists.2?);
        sp.stop_and_persist("✔", "Finished downloading metadata".into());

        let options = {
            let mut out = version_list.versions;
            if Confirm::new("Only stable versions?")
                .with_default(true)
                .prompt()?
            {
                out.retain(|v| v.stable);
            }
            out
        };
        let version = Select::new("Select version", options).prompt()?;

        let options = {
            let mut out = loader_list;
            if Confirm::new("Only stable loaders?")
                .with_default(true)
                .prompt()?
            {
                out.retain(|l| l.stable);
            }
            out
        };
        let loader = Select::new("Select loader", options).prompt()?;

        let options = {
            let mut out = installer_list;
            if Confirm::new("Only stable installers?")
                .with_default(true)
                .prompt()?
            {
                out.retain(|i| i.stable);
            }
            out
        };
        let installer = Select::new("Select installer", options).prompt()?;

        Ok(Self {
            installer: installer.version,
            loader: loader.version,
            version: version.version,
        })
    }

    async fn get_versions() -> Result<VersionList> {
        let url = "https://meta.fabricmc.net/v2/versions/";
        let res = reqwest::get(url).await?.error_for_status()?;
        let body = res.text().await?;
        let ver = serde_json::from_str(&body)?;
        Ok(ver)
    }

    async fn get_loaders() -> Result<Vec<LoaderInfo>> {
        let url = "https://meta.fabricmc.net/v2/versions/loader";
        let res = reqwest::get(url).await?.error_for_status()?;
        let body = res.text().await?;
        let ver = serde_json::from_str(&body)?;
        Ok(ver)
    }

    async fn get_installers() -> Result<Vec<InstallerInfo>> {
        let url = "https://meta.fabricmc.net/v2/versions/installer";
        let res = reqwest::get(url).await?.error_for_status()?;
        let body = res.text().await?;
        let ver = serde_json::from_str(&body)?;
        Ok(ver)
    }

    pub async fn install(&self, path: &Path) -> Result<()> {
        let url = format!(
            "https://meta.fabricmc.net/v2/versions/loader/{}/{}/{}/server/jar",
            self.version, self.loader, self.installer
        );
        let content = download_file(&url, "server.jar").await?;

        install_server_jar(path, &content).await?;

        Ok(())
    }
}

#[derive(Deserialize, Clone)]
struct VersionList {
    #[serde(rename = "game")]
    versions: Vec<VersionInfo>,
}

#[derive(Deserialize, Clone)]
struct VersionInfo {
    version: String,
    stable: bool,
}

impl Display for VersionInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let stable = match self.stable {
            true => "stable",
            false => "unstable",
        };
        write!(f, "{} - {}", self.version, stable)
    }
}

#[derive(Deserialize, Clone)]
struct LoaderInfo {
    version: String,
    stable: bool,
}

impl Display for LoaderInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let stable = match self.stable {
            true => "stable",
            false => "unstable",
        };
        write!(f, "{} - {}", self.version, stable)
    }
}

#[derive(Deserialize, Clone)]
struct InstallerInfo {
    version: String,
    stable: bool,
}

impl Display for InstallerInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let stable = match self.stable {
            true => "stable",
            false => "unstable",
        };
        write!(f, "{} - {}", self.version, stable)
    }
}
