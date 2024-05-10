use crate::api::{get_builds, get_versions};
use crate::error::*;
use inquire::{Select, Text};
use itertools::Itertools;
use serde::Deserialize;
use std::path::PathBuf;
use std::str::FromStr;
use strum::{Display, EnumIter, IntoEnumIterator};

#[derive(Debug)]
pub struct ServerInfo {
    pub path: PathBuf,
    pub distribution: Distribution,
    pub version: String,
    pub build_id: i64,
}

impl ServerInfo {
    pub async fn new() -> Result<Self> {
        let directory = Text::new("Select directory").prompt()?;
        let directory = PathBuf::from_str(&directory)?;

        let options = Distribution::iter().collect();
        let distribution = Select::new("Select distribution", options).prompt()?;

        let version_list = get_versions(distribution).await?;

        let mut options = version_list.versions;
        options.reverse();
        let version = Select::new("Select version", options).prompt()?;

        let build_list = get_builds(distribution, &version).await?;
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

        Ok(Self {
            path: directory,
            distribution,
            version,
            build_id,
        })
    }
}

#[derive(Debug, Display, Deserialize, EnumIter, Copy, Clone)]
pub enum Distribution {
    Paper,
    Folia,
    Velocity,
}

#[derive(Deserialize, Debug)]
pub struct BuildList {
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
