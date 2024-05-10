use crate::distribution::download_file;
use crate::error::*;
use bytes::Bytes;
use inquire::Select;
use serde::Deserialize;

pub struct Purpur {
    version: String,
}

impl Purpur {
    pub async fn new() -> Result<Self> {
        let version_list = Self::get_versions().await?;

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

    pub async fn download(&self) -> Result<Bytes> {
        let url = format!(
            "https://api.purpurmc.org/v2/purpur/{}/latest/download",
            self.version,
        );
        download_file(&url).await
    }
}

#[derive(Deserialize)]
struct VersionList {
    versions: Vec<String>,
}
