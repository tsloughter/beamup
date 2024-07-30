use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::io::Write;

use std::process;
use toml;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    install_dir: String,
    erlang: Option<Language>,
    gleam: Option<Language>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Language {
    default: Option<String>,
}

pub fn home_config_file() -> String {
    let config_dir = match dirs::config_dir() {
        Some(d) => d,
        None => {
            error!("no home directory available");
            process::exit(1)
        }
    };
    let cache_dir = match dirs::cache_dir() {
        Some(d) => d,
        None => {
            error!("no home directory available");
            process::exit(1)
        }
    };

    let default_config = config_dir.join("beamup").join("config");
    let default_cache = cache_dir.join("beamup");

    let _ = fs::create_dir_all(config_dir.join("beamup"));
    let _ = fs::create_dir_all(cache_dir.join("beamup"));

    if !default_config.exists() {
        let config = Config {
            install_dir: default_cache.to_str().unwrap().to_string(),
            erlang: None,
            gleam: None,
        };

        write_config(default_config.to_str().unwrap().to_string(), config).unwrap();
        info!(
            "Created a default config at {:?}",
            default_config.to_owned()
        );
    }

    default_config.to_str().unwrap().to_string()
}

pub fn home_config() -> (String, Config) {
    let config_file = home_config_file();
    (config_file.to_owned(), read_config(config_file))
}

pub fn read_config(file: String) -> Config {
    let toml_str = fs::read_to_string(file).expect("Failed to read config file");
    let config: Config = toml::from_str(toml_str.as_str()).unwrap();
    config
}

pub fn write_config(file_path: String, config: Config) -> io::Result<()> {
    let toml_string = toml::to_string(&config).unwrap();
    let mut file = fs::File::create(file_path)?;
    file.write_all(toml_string.as_bytes())?;
    Ok(())
}
