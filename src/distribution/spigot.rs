use crate::args::Args;
use crate::distribution::{install_eula, install_start_script};
use crate::error::*;
use crate::java::installed_versions;
use bytes::Bytes;
use futures_util::future::join3;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{Select, Text};
use spinners::{Spinner, Spinners};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub struct Spigot {
    version: String,
}

impl Spigot {
    pub async fn new() -> Result<Self> {
        let version;
        loop {
            let ver = Text::new("Select version").prompt()?;

            let mut sp = Spinner::new(Spinners::Dots, "Validating version".into());
            if !Self::check_version(&ver).await? {
                sp.stop_with_message("Version is invalid".to_string());
                continue;
            }
            sp.stop_with_message("Successfully validated version".to_string());
            version = ver;
            break;
        }

        Ok(Self { version })
    }

    pub async fn install(&self, path: &PathBuf, args: Args) -> Result<()> {
        // download buildtools
        let bytes = Self::download_build_tools().await?;
        let mut build_path = path.clone();
        build_path.push("build_cache");

        fs::create_dir_all(&build_path).await?;

        // buildtools.jar
        let mut tool_path = build_path.clone();
        tool_path.push("buildtools.jar");
        let mut build_tools = File::create(&tool_path).await?;
        build_tools.write_all(&bytes).await?;

        let java_path = if let Some(path) = args.java_path {
            PathBuf::from(&path)
        } else {
            let options = installed_versions()?;
            let java_version = Select::new("Select Java version", options).prompt()?;
            PathBuf::from(&java_version)
        };

        // run buildtools
        let mut sp = Spinner::new(
            Spinners::Dots,
            "Building server.jar. This takes a few minutes...".to_string(),
        );
        let output = Command::new(java_path.as_os_str())
            .current_dir(&build_path)
            .arg("-jar")
            .arg("buildtools.jar")
            .arg("--rev")
            .arg(&self.version)
            .output()?;
        if !output.status.success() {
            let mut sp = Spinner::new(Spinners::Dots, "Deleting temp files...".to_string());
            // SCARY
            // delete build cache
            fs::remove_dir_all(&build_path).await?;
            sp.stop_with_message("Finished deleting temp files".to_string());

            let err = String::from_utf8(output.stderr)?;
            return Err(Error::Other(format!(
                "Error while executing BuildTools: {}",
                err
            )));
        }
        sp.stop_with_message("Finished building".to_string());

        let mut server_path = build_path.clone();
        server_path.push(format!("spigot-{}.jar", self.version));
        let server_path = Path::new(&server_path);
        if !server_path.exists() {
            return Err(Error::Other("Error while executing BuildTools".to_string()));
        }

        let mut new_path = path.clone();
        new_path.push("server.jar");
        fs::copy(server_path, new_path).await?;

        let mut sp = Spinner::new(Spinners::Dots, "Deleting temp files...".to_string());
        // SCARY
        // delete build cache
        let del = fs::remove_dir_all(&build_path);
        let start = install_start_script(path, &java_path);
        let eula = install_eula(path);

        let res = join3(del, start, eula).await;
        res.0?;
        res.1?;
        res.2?;

        sp.stop_with_message("Finished deleting temp files".to_string());

        Ok(())
    }

    async fn check_version(version: &str) -> Result<bool> {
        let url = format!("https://hub.spigotmc.org/versions/{}.json", version);
        let client = reqwest::Client::new();
        let res = client
            .get(&url)
            .header("User-Agent", "Fuck you spigot")
            .send()
            .await?;
        let code = res.status();
        Ok(code.is_success())
    }

    async fn download_build_tools() -> Result<Bytes> {
        let client = reqwest::Client::new();

        let url = "https://hub.spigotmc.org/jenkins/job/BuildTools/api/json";
        let res = client
            .get(url)
            .header("User-Agent", "automc client")
            .send()
            .await?
            .error_for_status()?;
        let body = res.text().await?;

        let mut build_tools_url = gjson::get(&body, "builds.0.url").str().to_string();
        build_tools_url.push_str("artifact/target/BuildTools.jar");

        let res = client
            .get(build_tools_url)
            .header("User-Agent", "automc client")
            .send()
            .await?
            .error_for_status()?;

        if let Some(file_size) = res.content_length() {
            let pb = ProgressBar::new(file_size);
            pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("=> "));

            let mut content = Vec::with_capacity(file_size as usize);

            let mut stream = res.bytes_stream();
            while let Some(chunk) = stream.next().await {
                let bytes = chunk?.to_vec();
                pb.inc(bytes.len() as u64);
                content.extend(bytes);
            }
            pb.finish();

            Ok(Bytes::from(content))
        } else {
            let mut sp = Spinner::new(Spinners::Dots, "Downloading buildtools".to_string());
            let content = res.bytes().await?;
            sp.stop_and_persist("✔", "Finished downloading buildtools".to_string());

            Ok(content)
        }
    }
}
