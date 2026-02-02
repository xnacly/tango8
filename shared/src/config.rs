use serde::Deserialize;
use std::{collections::HashMap, fs};

/// defined in t8.toml
#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub verbose: bool,
    pub io: HashMap<String, Device>,
}

impl Config {
    pub fn from_toml() -> Result<Self, Box<dyn std::error::Error>> {
        let bytes = fs::read("t8.toml")?;
        Ok(toml::from_slice(&bytes)?)
    }
}

#[derive(Default, Debug, Deserialize)]
pub struct Device {
    pub addr: u8,
    pub file: String,
}
