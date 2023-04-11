use std::{path::PathBuf, fs::File};
use anyhow::Result;
use serde::{Serialize, Deserialize};
pub fn load_config(config_path: PathBuf) -> Result<Config> {
    let abs_path = std::fs::canonicalize(config_path)?;
    let f = File::open(abs_path)?;
    Ok(serde_yaml::from_reader(f)?)
}
pub fn load_config_from_bytes(config: Vec<u8>) -> Result<Config> {
    Ok(serde_yaml::from_slice(&config)?)
}
pub fn load_config_from_str(config: &str) -> Result<Config> {
    Ok(serde_yaml::from_str(config)?)
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub frontend: PathBuf,
    pub port: u16,
}


#[derive(Debug, Deserialize)]
pub struct BaryAppAttr {
    pub secret_key: String,
}