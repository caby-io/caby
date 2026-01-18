use std::env::var;

use crate::{Error, Result};

pub struct ConfigEnv {
    // required
    pub caby_home_path: String,

    // optional
    pub caby_config_path: Option<String>,
}

impl ConfigEnv {
    pub fn new() -> Result<Self> {
        return Ok(Self {
            caby_home_path: var("CABY_HOME_PATH")
                .map_err(|_| Error::from(Error::Generic("CABY_HOME_PATH".to_owned())))?,
            caby_config_path: var("CABY_CONFIG_PATH").ok(),
        });
        // if let Ok(var("CABY_HOME_PATH")
    }
}
