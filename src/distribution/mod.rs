use bytes::Bytes;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use strum::{Display, EnumIter};
use crate::error::*;

pub use paper::Paper;
pub use folia::Folia;
pub use velocity::Velocity;
pub use purpur::Purpur;

pub mod paper;
pub mod folia;
pub mod velocity;
pub mod purpur;
pub mod fabric;

#[derive(Debug, Display, Deserialize, EnumIter, Copy, Clone)]
pub enum Distribution {
    Paper,
    Folia,
    Velocity,
    Purpur
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