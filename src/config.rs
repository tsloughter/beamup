use color_eyre::{eyre::eyre, eyre::Report, eyre::Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::io::Write;
use std::path::*;
use std::process;

use crate::languages;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    install_dir: String,
    erlang: Option<LanguageConfig>,
    gleam: Option<LanguageConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct LanguageConfig {
    default: Option<String>,
    installs: toml::Table,
}

fn get_language_config(language: &languages::Language, config: &Config) -> LanguageConfig {
    match language {
        languages::Language::Gleam => config.gleam.clone().unwrap_or_default(),
        languages::Language::Erlang => config.erlang.clone().unwrap_or_default(),
    }
}

fn get_default(lc: &Option<LanguageConfig>) -> String {
    match lc {
        None => {
            error!("No install found for {:?}", lc);
            process::exit(1)
        }
        Some(lc) => match &lc.default {
            None => {
                error!("No default found for language");
                process::exit(1);
            }
            Some(default) => default.to_string(),
        },
    }
}

pub fn install_to_use(bin: &str) -> Result<String, Report> {
    let language = languages::bin_to_language(bin);
    let (_, config) = home_config();

    match language {
        languages::Language::Gleam => {
            let id = get_default(&config.gleam);
            lookup_install_by_id(id, config.gleam)
        }
        languages::Language::Erlang => {
            let id = get_default(&config.gleam);
            lookup_install_by_id(id, config.erlang)
        }
    }
}

fn lookup_install_by_id(id: String, lc: Option<LanguageConfig>) -> Result<String> {
    match lc {
        None => Err(eyre!("No config found")),
        Some(language_config) => match language_config.installs.get(&id) {
            None => Err(eyre!("No install found for id {id}")),
            Some(toml::Value::String(dir)) => Ok(dir.to_owned()),
            _ => Err(eyre!("Bad directory found in installs for id {id}")),
        },
    }
}

pub fn update_language_config(id: &String, dir: String, lc: LanguageConfig) -> LanguageConfig {
    let LanguageConfig {
        default: _,
        installs: mut table,
    } = lc;
    table.insert(id.clone(), toml::Value::String(dir));
    LanguageConfig {
        default: Some(id.to_owned()),
        installs: table.clone(),
    }
}

pub fn add_install(
    language: &languages::Language,
    id: &String,
    dir: String,
    config_file: String,
    config: Config,
) {
    debug!("adding install {id} pointing to {dir}");
    let language_config = get_language_config(language, &config);

    let updated_language_config = update_language_config(id, dir, language_config.clone());

    let new_config = Config {
        gleam: Some(updated_language_config),
        ..config
    };

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
