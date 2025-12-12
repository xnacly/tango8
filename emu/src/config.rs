use serde::Deserialize;
use std::collections::HashMap;

/// defined in t8.toml
#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub verbose: bool,
    pub io: HashMap<String, Device>,
}

#[derive(Default, Debug, Deserialize)]
pub struct Device {
    pub addr: u8,
    pub file: String,
}
