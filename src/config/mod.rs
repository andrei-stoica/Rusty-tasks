extern crate serde;
extern crate serde_json;

use figment::providers::{Env, Format, Json, Serialized};
use figment::Figment;
use serde::{Deserialize, Serialize};
use std::env::var;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub editor: Option<String>,
    pub sections: Option<Vec<String>>,
    pub notes_dir: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            editor: Some("nano".into()),
            sections: Some(vec!["Daily".into(), "Weekly".into(), "Monthly".into()]),
            notes_dir: Some("Notes".into()),
        }
    }
}

impl Config {
    pub fn load(cfg_file: &str) -> Result<Self, &'static str> {
        Figment::from(Serialized::defaults(Config::default()))
            .merge(Env::raw().only(&["EDITOR"]))
            .merge(Json::file(cfg_file))
            .extract()
            .or(Err("Could not load config"))
    }

    pub fn write_default(cfg_file: &str) -> Result<(), &'static str> {
        let buf = serde_json::to_string_pretty(&Self::default())
            .or_else(|_| return Err("could not serialize default config"))?;

        let mut f = File::create(cfg_file).or_else(|_| Err("Could not open config file"))?;
        f.write_all(&buf.as_bytes())
            .or_else(|_| return Err("could not write default config to file"))?;

        Ok(())
    }

    pub fn expected_locations() -> Result<Vec<PathBuf>, &'static str> {
        let cfg_name = "rusty_task.json";
        let home = var("HOME").or(Err("$HOME environment variable not set"))?;
        let pwd = var("PWD").or(Err("$PWD environment variable not set"))?;

        let mut home_config_cfg = PathBuf::from(home.clone());
        home_config_cfg.push(".config");
        home_config_cfg.push(cfg_name);

        let mut home_cfg = PathBuf::from(home.clone());
        home_cfg.push(format!(".{}", cfg_name));

        let mut pwd_cfg = PathBuf::from(pwd.clone());
        pwd_cfg.push(format!(".{}", cfg_name));

        Ok(vec![home_config_cfg, home_cfg, pwd_cfg])
    }
}
