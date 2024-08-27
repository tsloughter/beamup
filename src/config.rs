use color_eyre::{eyre::eyre, eyre::Report, eyre::Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::*;

use crate::languages;

static LOCAL_CONFIG_FILE: &str = ".beamup.toml";
static CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    install_dir: String,
    erlang: Option<LanguageConfig>,
    gleam: Option<LanguageConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct LanguageConfig {
    default: Option<String>,
    default_build_options: Option<String>,
    installs: toml::Table,
}

pub fn print_ids(config: &Config) {
    println!("Erlang:");
    config.erlang.as_ref().map(print_language_ids);
    println!("");
    println!("Gleam:");
    config.gleam.as_ref().map(print_language_ids);
}

fn print_language_ids(lc: &LanguageConfig) {
    for id in lc.installs.keys() {
        println!("{id}")
    }
}

fn get_language_config(language: &languages::Language, config: &Config) -> LanguageConfig {
    match language {
        languages::Language::Gleam => config.gleam.clone().unwrap_or_default(),
        languages::Language::Erlang => config.erlang.clone().unwrap_or_default(),
    }
}

fn get_default_id(lc: &Option<LanguageConfig>) -> Result<String> {
    match lc {
        None => Err(eyre!("No default found for {:?}", lc)),
        Some(lc) => match &lc.default {
            None => Err(eyre!("No default found for language")),
            Some(default) => {
                debug!("Found default {:?}", default);
                Ok(default.to_string())
            }
        },
    }
}

pub fn switch(language: &languages::Language, id: &str, config: &Config) -> Result<()> {
    let language_config = get_language_config(language, config);

    // we just look it up to return an error if it doesn't exist
    let _ = lookup_install_by_id(id.to_string(), Some(language_config))?;

    let mut c = match local_config() {
        None => toml::Table::new(),
        Some(local_config) => local_config.clone(),
    };

    c.insert(language.to_string(), toml::Value::String(id.to_string()));

    let toml_string = toml::to_string(&c).unwrap();
    let mut file = fs::File::create(LOCAL_CONFIG_FILE)?;
    file.write_all(toml_string.as_bytes())?;
    Ok(())
}

fn get_local_id(language_str: String, local_config: &Option<toml::Table>) -> Option<&toml::Value> {
    match local_config {
        None => None,
        Some(lc) => lc.get(language_str.clone().as_str()),
    }
}

pub fn install_to_use(bin: &str) -> Result<String> {
    let language = languages::bin_to_language(bin)?;
    let (_, config) = home_config()?;
    let language_config = get_language_config(language, &config);
    let local_config = local_config();
    let language_str = language.to_string();

    let maybe_id = match get_local_id(language_str, &local_config) {
        None => None,
        Some(toml::Value::String(id)) => {
            debug!("Using id from local config file");
            Some(id)
        }
        _ => None,
    };

    let id = match maybe_id {
        None => {
            debug!("No local config found. Using global config");
            match language {
                languages::Language::Gleam => get_default_id(&config.gleam)?,
                languages::Language::Erlang => get_default_id(&config.erlang)?,
            }
        }
        Some(id) => id.clone(),
    };

    lookup_install_by_id(id, Some(language_config))
}

fn lookup_install_by_id(id: String, lc: Option<LanguageConfig>) -> Result<String> {
    debug!("Looking up install for {}", id);
    match lc {
        None => Err(eyre!("No config found")),
        Some(language_config) => match language_config.installs.get(&id) {
            None => Err(eyre!("No install found for id {id}")),
            Some(toml::Value::String(dir)) => {
                debug!("Found install in directory {}", dir);
                Ok(dir.to_owned())
            }
            _ => Err(eyre!("Bad directory found in installs for id {id}")),
        },
    }
}

pub fn lookup_default_build_options(language: &languages::Language, config: &Config) -> String {
    debug!("Looking up default configure options for {:?}", language);

    let lc = get_language_config(language, config);

    match lc.default_build_options {
        None => "".to_string(),
        Some(options) => options.to_owned(),
    }
}

pub fn set_default(
    language: &languages::Language,
    id: &String,
    config_file: String,
    config: Config,
) -> Result<(), Report> {
    debug!("set default {:?} to use to {:?}", language, id);
    let lc = get_language_config(language, &config);
    let LanguageConfig {
        default: _,
        installs: installs_table,
        default_build_options,
    } = lc;

    let new_lc = LanguageConfig {
        default: Some(id.to_owned()),
        installs: installs_table.clone(),
        default_build_options: default_build_options.clone(),
    };

    let new_config = match language {
        languages::Language::Gleam => Config {
            gleam: Some(new_lc),
            ..config
        },
        languages::Language::Erlang => Config {
            erlang: Some(new_lc),
            ..config
        },
    };

    write_config(config_file, new_config)
}

pub fn update_language_config(id: &String, dir: String, lc: LanguageConfig) -> LanguageConfig {
    let LanguageConfig {
        default: _,
        installs: mut table,
        default_build_options,
    } = lc;
    table.insert(id.clone(), toml::Value::String(dir));
    LanguageConfig {
        default: Some(id.to_owned()),
        installs: table.clone(),
        default_build_options: default_build_options.clone(),
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

    let new_config = match language {
        languages::Language::Gleam => Config {
            gleam: Some(updated_language_config),
            ..config
        },
        languages::Language::Erlang => Config {
            erlang: Some(updated_language_config),
            ..config
        },
    };

    let _ = write_config(config_file, new_config);
}

pub fn language_release_dir(
    language: &languages::Language,
    id: &String,
    force: &Option<bool>,
) -> Result<PathBuf> {
    let data_dir = dirs::data_local_dir();
    let release_dir = data_dir
        .unwrap()
        .join("beamup")
        .join(language.to_string())
        .join(id);

    match release_dir.try_exists() {
        Ok(true) =>
            match force {
                Some(true) => {
                    info!("Force enabled. Deleting existing release directory {release_dir:?}");
                    fs::remove_dir_all(&release_dir)?
                },
                _ => return Err(eyre!("Release directory for id {id:} already exists. Use `-f` to delete {release_dir:?} and recreate instead of giving this error.")),
            }
        Ok(false) => {},
        Err(e) => return Err(eyre!(
            "Unable to check for existence of release directory for id {id}: {e:?}"
        )),
    };

    let _ = std::fs::create_dir_all(&release_dir);

    Ok(release_dir)
}

pub fn bin_dir() -> PathBuf {
    match dirs::executable_dir() {
        Some(bin_dir) => bin_dir,
        None => {
            let home_dir = dirs::home_dir().unwrap();
            Path::new(&home_dir).join(".beamup").join("bin")
        }
    }
}

pub fn home_config_file() -> Result<String> {
    let config_dir = match dirs::config_local_dir() {
        Some(d) => d,
        None => return Err(eyre!("no home directory available")),
    };
    let data_dir = match dirs::data_local_dir() {
        Some(d) => d,
        None => return Err(eyre!("no home directory available")),
    };

    let default_config = config_dir.join("beamup").join(CONFIG_FILE);
    let default_data = data_dir.join("beamup");

    let _ = fs::create_dir_all(config_dir.join("beamup"));
    let _ = fs::create_dir_all(data_dir.join("beamup"));

    if !default_config.exists() {
        let config = Config {
            install_dir: default_data.to_str().unwrap().to_string(),
            erlang: Some(LanguageConfig {
                default: None,
                installs: toml::Table::new(),
                default_build_options: None,
            }),
            gleam: Some(LanguageConfig {
                default: None,
                installs: toml::Table::new(),
                default_build_options: None,
            }),
        };

        write_config(default_config.to_str().unwrap().to_string(), config)?;
        info!(
            "Created a default config at {:?}",
            default_config.to_owned()
        );
    }

    Ok(default_config.to_str().unwrap().to_string())
}

fn local_config() -> Option<toml::Table> {
    match fs::read_to_string(LOCAL_CONFIG_FILE) {
        Ok(local_config_str) => toml::from_str(local_config_str.as_str()).ok(),
        _ => None,
    }
}

pub fn home_config() -> Result<(String, Config)> {
    let config_file = home_config_file()?;
    Ok((config_file.to_owned(), read_config(config_file)))
}

pub fn read_config(file: String) -> Config {
    let toml_str = fs::read_to_string(file).expect("Failed to read config file");
    let config: Config = toml::from_str(toml_str.as_str()).unwrap();
    config
}

pub fn write_config(file_path: String, config: Config) -> Result<()> {
    let toml_string = toml::to_string(&config).unwrap();
    let mut file = fs::File::create(file_path)?;
    file.write_all(toml_string.as_bytes())?;
    Ok(())
}
