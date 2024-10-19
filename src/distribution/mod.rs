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
pub use spigot::Spigot;
pub use vanilla::Vanilla;
pub use velocity::Velocity;

mod fabric;
mod folia;
mod paper;
mod purpur;
mod spigot;
mod vanilla;
mod velocity;

#[derive(Debug, Display, Deserialize, EnumIter, Copy, Clone)]
pub enum Distribution {
    Paper,
    Purpur,
    Velocity,
    Folia,
    Spigot,
    Fabric,
    Vanilla,
}

pub async fn download_file(url: &str, message: &str) -> Result<Bytes> {
    let req = reqwest::get(url).await?.error_for_status()?;

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
            content.extend(bytes);
        }
        pb.finish();
        Ok(Bytes::from(content))
    } else {
        let mut sp = Spinner::new(Spinners::Dots, format!("Downloading {}", message));
        let content = req.bytes().await?;
        sp.stop_and_persist("✔", format!("Finished downloading {}", message));

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
        #[cfg(target_os = "windows")]
        {
            path.push("start.bat");
            let mut file = File::create(path).await?;
            file.write_all(
                format!("\"{}\" -jar server.jar", java_path.to_str().unwrap()).as_bytes(),
            )
            .await?;
        }
    } else if cfg!(unix) {
        #[cfg(target_os = "linux")]
        {
            println!("pre everything");
            use std::os::unix::fs::PermissionsExt;
            path.push("start.sh");
            println!("after path push");
            let mut file = File::create(&path).await?;
            println!("after file create");
            file.write_all(
                format!(
                    "#!/usr/bin/env sh\n{} -jar server.jar -nogui",
                    java_path.to_str().unwrap()
                )
                .as_bytes(),
            )
            .await?;
            println!("after file write");

            let mut perms = file.metadata().await?.permissions();
            perms.set_mode(0o755); // same as chmod +x
            fs::set_permissions(path, perms)?;
            println!("after perms");
        }
    } else {
        return Err(Error::Other("unsupported OS".to_string()));
    }

    Ok(())
}
