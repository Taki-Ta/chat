use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::{env, fs::File};

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    pub port: u16,
    pub db_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthConfig {
    pub pk: String,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        //read from ./notify.yaml or  /etc/config/notify.yaml or from env NOTIFY_CONFIG
        let ret = match (
            File::open("notify.yaml"),
            File::open("/etc/config/notify.yaml"),
            env::var("NOTIFY_CONFIG"),
        ) {
            (Ok(f), _, _) => serde_yaml::from_reader(f),
            (_, Ok(f), _) => serde_yaml::from_reader(f),
            (_, _, Ok(path)) => serde_yaml::from_reader(File::open(path)?),
            _ => bail!("No config file found"),
        };
        Ok(ret?)
    }
}
