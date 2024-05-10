use std::fs;
use crate::error::*;
use std::process::Command;

pub fn installed_versions() -> Result<Vec<String>> {
    return if cfg!(windows) {
        let output = Command::new("where").arg("java").output()?;
        let versions = output
            .stdout
            .into_iter()
            .fold(String::new(), |acc, c| acc + &(c as char).to_string())
            .split_terminator("\r\n")
            .map(|str| str.to_string())
            .collect();

        Ok(versions)
    } else if cfg!(unix) {
        let mut versions = Vec::new();

        for entry in fs::read_dir("/usr/lib/jvm")? {
            let entry = entry?;

            if entry.path().is_dir() && !entry.path().is_symlink() {
                let mut path = entry.path();
                path.push("bin");
                path.push("java");
                versions.push(path.to_str().unwrap().to_string())
            }
        }
        if versions.is_empty() {
            return Err(Error::Other("No Java versions where found".to_string()));
        }

        Ok(versions)
    } else {
        Err(Error::Other("unsupported OS".to_string()))
    };
}
