use crate::error::*;
use bytes::Bytes;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use spinners::{Spinner, Spinners};
use std::fs;
use std::path::{Path, PathBuf};
use strum::{Display, EnumIter};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

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

    if let Some(file_size) = req.content_length() {
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
    } else {
        let mut sp = Spinner::new(Spinners::Dots, "Downloading server.jar".into());
        let content = req.bytes().await?;
        sp.stop_and_persist("✔", "Finished downloading server.jar".into());

        Ok(content)
    }
}

pub async fn install_eula(path: &PathBuf) -> Result<()> {
    fs::create_dir_all(path)?;

    let mut path = path.to_owned();
    path.push("eula.txt");

    let mut eula = File::create(path).await?;
    eula.write_all(b"eula=true").await?;

    Ok(())
}

pub async fn install_server_jar(path: &PathBuf, bytes: &Bytes) -> Result<()> {
    fs::create_dir_all(path)?;

    let mut path = path.to_owned();
    path.push("server.jar");

    let mut server_jar = File::create(&path).await?;
    server_jar.write_all(bytes).await?;

    Ok(())
}

pub async fn install_start_script(path: &PathBuf, java_path: &Path) -> Result<()> {
    let mut path = path.to_owned();

    if cfg!(windows) {
        path.push("start.bat");
        let mut file = File::create(path).await?;
        file.write_all(
            format!("\"{}\" -jar server.jar -nogui", java_path.to_str().unwrap()).as_bytes(),
        )
        .await?;
    } else if cfg!(unix) {
        path.push("start.sh");
        let mut file = File::create(path).await?;
        file.write_all(
            format!(
                "#!/usr/bin/env sh\n{} -jar server.jar -nogui",
                java_path.to_str().unwrap()
            )
            .as_bytes(),
        )
        .await?;
    } else {
        return Err(Error::Other("unsupported OS".to_string()));
    }

    Ok(())
}
