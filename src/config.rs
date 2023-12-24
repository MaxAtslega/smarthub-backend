use std::{env, fs};
use std::net::IpAddr;

use log::{debug, error};
use serde_derive::Deserialize;
use thiserror::Error;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub app: AppConf,
    pub websocket: WebSocketConf,
    pub log: LogConf,
}

#[derive(Deserialize, Debug)]
pub struct LogConf {
    pub file: String,
}

#[derive(Deserialize, Debug)]
pub struct AppConf {
    pub environment: String,
}

#[derive(Deserialize, Debug)]
pub struct WebSocketConf {
    pub address: IpAddr,
    pub port: u16
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Config File could not be found")]
    ConfigNotFound(std::io::Error),

    #[error("Config File could not be found")]
    ParsingError(toml::de::Error),

    #[error("Env variable could not be found")]
    EnvVarNotFound(std::env::VarError),
}

impl Config {
    pub fn from_any() -> Result<Self, ConfigError> {
        //Try to read from path env var
        let env_result = Self::from_env_path();
        match env_result {
            Ok(config) => {
                debug!("Loaded config from env path");
                return Ok(config);
            }
            Err(error) => {
                debug!("Could not load config from env path: {}", error);
            }
        }

        //Try to read default path
        let default_result = Self::from_default_path();
        match default_result {
            Ok(config) => {
                debug!("Loaded config from default path");
                return Ok(config);
            }
            Err(error) => {
                error!("Could not load config: {}", error);
                return Err(error);
            }
        }
    }

    // Read Config from default path
    pub fn from_default_path() -> Result<Self, ConfigError> {
        let path = "config.toml";
        Self::from_file_path(&path)
    }

    // Read Config from path in CONFIG_LOCATION env variable
    pub fn from_env_path() -> Result<Self, ConfigError> {
        let path = env::var("CONFIG_LOCATION")
            .map_err(|e| ConfigError::EnvVarNotFound(e))?;
        Self::from_file_path(&path)
    }

    // Read and Parse Config from path
    pub fn from_file_path(path: &str) -> Result<Self, ConfigError> {
        let data = fs::read_to_string(path)
            .map_err(|e| ConfigError::ConfigNotFound(e))?;

        toml::from_str(data.as_str())
            .map_err(|e| ConfigError::ParsingError(e))
    }
}
