use crate::error::*;
use java_locator::locate_java_home;
use std::io::BufRead;
use std::path::PathBuf;
use std::process::Command;

#[cfg(windows)]
pub fn java_versions() -> Result<Vec<PathBuf>> {
    let output = Command::new("where").arg("java").output()?;

    if !output.status.success() {
        return Err(Error::Other("error running where command".to_string()));
    }

    let mut java_versions = output
        .stdout
        .lines()
        .filter_map(|line| line.ok().map(PathBuf::from))
        .collect::<Vec<PathBuf>>();

    if java_versions.is_empty() {
        let mut java_home = PathBuf::from(locate_java_home()?);
        java_home.push("bin");
        java_home.push("java.exe");

        java_versions.push(java_home);
    }

    Ok(java_versions)
}

#[cfg(unix)]
pub fn java_versions() -> Result<Vec<PathBuf>> {
    let output = Command::new("which").arg("java").output()?;

    if !output.status.success() {
        return Err(Error::Other("error running which command".to_string()));
    }

    let mut java_versions = output
        .stdout
        .lines()
        .filter_map(|line| line.ok().map(PathBuf::from))
        .collect::<Vec<PathBuf>>();

    if java_versions.is_empty() {
        let mut java_home = PathBuf::from(locate_java_home()?);
        java_home.push("bin");
        java_home.push("java");

        java_versions.push(java_home);
    }

    Ok(java_versions)
}
