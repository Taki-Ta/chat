use std::{env, fs::File};
use anyhow::bail;
use serde::{Serialize,Deserialize};
#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub server: ServerConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl AppConfig{
    pub fn load()->anyhow::Result<Self>{
        //read from ./app.yml or  /etc/config/app.yml or from env CHAT_CONFIG
        let ret=match (
            File::open("app.yml"),
            File::open("/etc/config/app.yml"),
            env::var("CHAT_CONFIG")
        ){
            (Ok(f),_,_)=>serde_yaml::from_reader(f),
            (_,Ok(f),_)=>serde_yaml::from_reader(f),
            (_,_,Ok(path))=>serde_yaml::from_reader(File::open(path)?),
            _=>bail!("No config file found")
        };
        Ok(ret?)
    }
}