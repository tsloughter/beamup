use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::io::Write;
use std::path::*;
use std::process;
use toml;

use crate::languages;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    install_dir: String,
    erlang: Option<LanguageConfig>,
    gleam: Option<LanguageConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LanguageConfig {
    default: Option<String>,
    installs: toml::Table,
}

// #[derive(Debug, Deserialize, Serialize)]
// pub struct Install {}

pub fn install_to_use() -> &'static str {
    "/home/tristan/.cache/beamup/gleam/latest"
}

pub fn add_install(
    language: &languages::Language,
    id: String,
    dir: String,
    config_file: String,
    config: Config,
) {
    debug!("adding install {id} pointing to {dir}");
    let language_config = match language {
        languages::Language::Gleam =>
            update_language_config(config.gleam)



            match config.gleam {
            None => {
                let table: &mut toml::Table = &mut toml::Table::new();
                table.insert(id.clone(), toml::Value::String(dir));
                LanguageConfig {
                    default: Some(id),
                    installs: table.clone(),
                }
            }
            Some(LanguageConfig {
                default: _,
                installs: mut table,
            }) => {
                table.insert(id.clone(), toml::Value::String(dir));
                LanguageConfig {
                    default: Some(id),
                    installs: table.clone(),
                }
            }
        },
        languages::Language::Erlang => LanguageConfig {
            default: None,
            installs: toml::Table::new(),
        },
    };

    debug!("CONFIG: {:?}", language_config);

    let new_config = Config {
        gleam: Some(language_config),
        ..config
    };

    debug!("CONFIG: {:?}", new_config);

    let _ = write_config(config_file, new_config);
}

pub fn language_release_dir(language: languages::Language, id: String) -> PathBuf {
    let cache_dir = dirs::cache_dir();
    let release_dir = cache_dir
        .unwrap()
        .join("beamup")
        .join(language.to_string())
        .join(id);

    let _ = std::fs::create_dir_all(&release_dir);

    release_dir
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
            erlang: Some(LanguageConfig {
                default: None,
                installs: toml::Table::new(),
            }),
            gleam: Some(LanguageConfig {
                default: None,
                installs: toml::Table::new(),
            }),
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
