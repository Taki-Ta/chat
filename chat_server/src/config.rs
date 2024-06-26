use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::{env, fs::File, path::PathBuf};
#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    pub port: u16,
    pub db_url: String,
    pub base_url: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthConfig {
    pub sk: String,
    pub pk: String,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        //read from ./chat.yaml or  /etc/config/chat.yaml or from env CHAT_CONFIG
        let ret = match (
            File::open("chat.yaml"),
            File::open("/etc/config/chat.yaml"),
            env::var("CHAT_CONFIG"),
        ) {
            (Ok(f), _, _) => serde_yaml::from_reader(f),
            (_, Ok(f), _) => serde_yaml::from_reader(f),
            (_, _, Ok(path)) => serde_yaml::from_reader(File::open(path)?),
            _ => bail!("No config file found"),
        };
        Ok(ret?)
    }
}
