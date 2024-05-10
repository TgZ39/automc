use crate::error::*;
use bytes::Bytes;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use strum::{Display, EnumIter};

pub use fabric::Fabric;
pub use folia::Folia;
pub use paper::Paper;
pub use purpur::Purpur;
pub use velocity::Velocity;

pub mod fabric;
pub mod folia;
pub mod paper;
pub mod purpur;
pub mod velocity;

#[derive(Debug, Display, Deserialize, EnumIter, Copy, Clone)]
pub enum Distribution {
    Paper,
    Folia,
    Velocity,
    Purpur,
    Fabric,
}

pub async fn download_file(url: &str) -> Result<Bytes> {
    let req = reqwest::get(url).await?;
    let file_size = req.content_length().unwrap();

    let pb = ProgressBar::new(file_size);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("=> "));

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
