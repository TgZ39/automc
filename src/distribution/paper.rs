use crate::args::Args;
use crate::distribution::{download_file, install_eula, install_server_jar, install_start_script};
use crate::error::*;
use crate::java::installed_versions;
use inquire::Select;
use itertools::Itertools;
use serde::Deserialize;
use spinners::{Spinner, Spinners};
use std::path::PathBuf;
use strum::Display;

pub struct Paper {
    version: String,
    build_id: i64,
}

impl Paper {
    pub async fn new() -> Result<Self> {
        let mut sp = Spinner::new(Spinners::Dots, "Downloading metadata".into());
        let version_list = Self::get_versions().await?;
        sp.stop_and_persist("✔", "Finished downloading metadata".into());

        let mut options = version_list.version_groups;
        options.reverse();
        let group = Select::new("Select version group", options).prompt()?;

        let mut options = version_list.versions;
        options.retain(|v| v.starts_with(&group));
        options.reverse();
        let version = Select::new("Select version", options).prompt()?;

        let mut sp = Spinner::new(Spinners::Dots, "Downloading build metadata".into());
        let build_list = Self::get_builds(&version).await?;
        sp.stop_and_persist("✔", "Finished downloading build metadata".into());

        let options = build_list
            .builds
            .iter()
            .unique_by(|b| b.channel)
            .map(|b| b.channel)
            .collect::<Vec<Channel>>();
        let channel = Select::new("Select channel", options).prompt()?;
        let build_id = build_list
            .builds
            .iter()
            .rfind(|&b| b.channel == channel)
            .unwrap()
            .build_id;

        Ok(Self { version, build_id })
    }

    async fn get_versions() -> Result<VersionList> {
        let url = "https://api.papermc.io/v2/projects/paper";
        let res = reqwest::get(url).await?.error_for_status()?;
        let body = res.text().await?;
        let ver = serde_json::from_str(&body)?;
        Ok(ver)
    }

    async fn get_builds(version: &str) -> Result<BuildList> {
        let url = format!(
            "https://api.papermc.io/v2/projects/paper/versions/{}/builds",
            version
        );
        let res = reqwest::get(url).await?.error_for_status()?;
        let body = res.text().await?;
        let builds = serde_json::from_str::<BuildList>(&body)?;

        Ok(builds)
    }

    pub async fn install(&self, path: &PathBuf, args: Args) -> Result<()> {
        let jar_name = format!("paper-{}-{}.jar", self.version, self.build_id);
        let url = format!(
            "https://api.papermc.io/v2/projects/paper/versions/{}/builds/{}/downloads/{}",
            self.version, self.build_id, jar_name
        );
        let content = download_file(&url, "server.jar").await?;

        install_server_jar(path, &content).await?;
        install_eula(path).await?;

        let java_path = if let Some(path) = args.java_path {
            PathBuf::from(&path)
        } else {
            let options = installed_versions()?;
            let java_version = Select::new("Select Java version", options).prompt()?;
            PathBuf::from(&java_version)
        };
        install_start_script(path, &java_path).await?;

        Ok(())
    }
}

#[derive(Deserialize)]
struct VersionList {
    version_groups: Vec<String>,
    versions: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct BuildList {
    builds: Vec<BuildInfo>,
}

#[derive(Deserialize, Debug)]
struct BuildInfo {
    #[serde(rename = "build")]
    build_id: i64,
    channel: Channel,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Display, Copy, Clone, Hash)]
enum Channel {
    #[serde(rename = "experimental")]
    Experimental,
    #[serde(rename = "default")]
    Default,
}
