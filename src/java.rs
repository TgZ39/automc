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
        todo!()
    } else {
        Err(Error::Other("unsupported OS".to_string()))
    };
}
