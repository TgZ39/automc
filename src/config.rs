use crate::error::*;
use serde::{Deserialize, Serialize};

const APP_NAME: &str = "automc";
const CONFIG_NAME: &str = "config";

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Config {
    pub accepted_eula: bool,
}

impl Config {
    pub fn load() -> Result<Self> {
        Ok(confy::load(APP_NAME, Some(CONFIG_NAME))?)
    }

    pub fn save(&self) -> Result<()> {
        Ok(confy::store(APP_NAME, Some(CONFIG_NAME), self)?)
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        self.save().expect("error saving config")
    }
}
