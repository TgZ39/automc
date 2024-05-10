use crate::error::*;
use crate::server_info::{BuildList, Distribution};
use bytes::Bytes;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;

pub async fn get_versions(distribution: Distribution) -> Result<VersionList> {
    let url = format!(
        "https://api.papermc.io/v2/projects/{}",
        distribution.to_string().to_lowercase()
    );
    let res = reqwest::get(url).await?;
    let body = res.text().await?;
    let ver = serde_json::from_str(&body)?;
    Ok(ver)
}

#[derive(Deserialize, Debug)]
pub struct VersionList {
    #[serde(rename = "project_name")]
    pub project: Distribution,
    pub version_groups: Vec<String>,
    pub versions: Vec<String>,
}

pub async fn get_builds(distribution: Distribution, version: &str) -> Result<BuildList> {
    let url = format!(
        "https://api.papermc.io/v2/projects/{}/versions/{}/builds",
        distribution.to_string().to_lowercase(),
        version
    );
    let res = reqwest::get(url).await?;
    let body = res.text().await?;
    let builds = serde_json::from_str::<BuildList>(&body)?;

    Ok(builds)
}

pub async fn download(distribution: Distribution, version: &str, build: i64) -> Result<Bytes> {
    let jar_name = format!(
        "{}-{}-{}.jar",
        distribution.to_string().to_lowercase(),
        version,
        build
    );
    let url = format!(
        "https://api.papermc.io/v2/projects/{}/versions/{}/builds/{}/downloads/{}",
        distribution.to_string().to_lowercase(),
        version,
        build,
        jar_name
    );
    let req = reqwest::get(url).await?;
    let file_size = req.content_length().unwrap();

    let pb = ProgressBar::new(file_size);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));

    let mut content = Vec::with_capacity(file_size as usize);

    let mut stream = req.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let bytes = chunk?.to_vec();
        pb.inc(bytes.len() as u64);
        content.extend(bytes)
    }
    pb.finish_with_message("{msg} done downloading");
    Ok(Bytes::from(content))
}
