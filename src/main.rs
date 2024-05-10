use crate::api::download;
use crate::server_info::ServerInfo;
use error::*;
use inquire::Text;
use std::fs::File;
use std::io::Write;
use tokio::fs;

pub mod api;
mod error;
pub mod server_info;

#[tokio::main]
async fn main() -> Result<()> {
    let info = ServerInfo::new().await?;
    let content = download(info.distribution, &info.version, info.build_id).await?;

    fs::create_dir_all(&info.path).await?;

    let mut path = info.path.clone();
    path.push("server.jar");
    let mut server_jar = File::create(path)?;
    server_jar.write_all(&content)?;

    let mut path = info.path.clone();
    path.push("eula.txt");
    let mut eula = File::create(path)?;
    eula.write_all(b"eula=true")?;

    Text::new("Press <ENTER> to exit...").prompt()?;
    Ok(())
}
