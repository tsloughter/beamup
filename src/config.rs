use crate::components;
use crate::languages;
use color_eyre::{eyre::eyre, eyre::Report, eyre::Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::*;

static LOCAL_CONFIG_FILE: &str = ".beamup.toml";
static CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    install_dir: String,
    erlang: Option<LanguageConfig>,
    gleam: Option<LanguageConfig>,
    elixir: Option<LanguageConfig>,
    elp: Option<ComponentConfig>,
    rebar3: Option<ComponentConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct LanguageConfig {
    default: Option<String>,
    default_build_options: Option<String>,
    installs: toml::Table,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ComponentConfig {
    default: Option<String>,
    default_build_options: Option<String>,
    installs: toml::Table,
}

pub fn print_ids(config: &Config) {
    println!("Elixir:");
    config.elixir.as_ref().map(print_language_ids);
    println!();
    println!("Erlang:");
    config.erlang.as_ref().map(print_language_ids);
    println!();
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
        languages::Language::Elixir => config.elixir.clone().unwrap_or_default(),
    }
}

fn get_component_config(kind: &components::Kind, config: &Config) -> ComponentConfig {
    match kind {
        components::Kind::Elp => config.elp.clone().unwrap_or_default(),
        components::Kind::Rebar3 => config.rebar3.clone().unwrap_or_default(),
    }
}

fn get_default_id(lc: &Option<LanguageConfig>) -> Result<String> {
    match lc {
        None => Err(eyre!("No default found for language {:?}", lc)),
        Some(lc) => match &lc.default {
            None => Err(eyre!("No default found for language {:?}", lc)),
            Some(default) => {
                debug!("Found default {:?}", default);
                Ok(default.to_string())
            }
        },
    }
}

fn get_component_default_id(c: &Option<ComponentConfig>) -> Result<String> {
    match c {
        None => Err(eyre!("No default found for component {:?}", c)),
        Some(c) => match &c.default {
            None => Err(eyre!("No default found for component {:?}", c)),
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

pub fn get_otp_major_vsn() -> Result<String> {
    let dir = match install_to_use("erl") {
        Ok(dir) => Ok(dir),
        Err(_) => Err(eyre!("No default Erlang installation found. Install an Erlang version, like `beamup install erlang latest` or set a default with `beamup default erlang <ID>` first.")),
    }?;
    let releases_dir = Path::new(&dir).join("lib").join("erlang").join("releases");

    for entry in std::fs::read_dir(&releases_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let binding = entry.file_name();
            let otp_major_vsn = binding
                .to_str()
                .ok_or(eyre!("Unable to convert OTP vsn {binding:?} to string"))?;

            return Ok(otp_major_vsn.to_string());
        }
    }

    Err(eyre!("No installed OTP release found in {releases_dir:?}"))
}

pub fn component_install_to_use(kind: &components::Kind) -> Result<String> {
    let (_, config) = home_config()?;
    let component_config = get_component_config(kind, &config);
    let local_config = local_config();
    let component_str = kind.to_string();

    let maybe_id = match get_local_id(component_str, &local_config) {
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
            match kind {
                components::Kind::Elp => get_component_default_id(&config.elp)?,
                components::Kind::Rebar3 => get_component_default_id(&config.rebar3)?,
            }
        }
        Some(id) => id.clone(),
    };

    lookup_component_install_by_id(id, Some(component_config))
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
                languages::Language::Elixir => get_default_id(&config.elixir)?,
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
            // backwards compatible clause for when id's only pointed
            // to a directory and not more metadata
            Some(toml::Value::String(dir)) => {
                debug!("Found install in directory {}", dir);
                Ok(dir.to_owned())
            }
            Some(t @ toml::Value::Table(_)) => {
                if let Some(toml::Value::String(dir)) = t.get("dir") {
                    Ok(dir.to_string())
                } else {
                    Err(eyre!("No directory found for install id {id}"))
                }
            }
            _ => Err(eyre!("Bad directory found in installs for id {id}")),
        },
    }
}

fn lookup_component_install_by_id(id: String, lc: Option<ComponentConfig>) -> Result<String> {
    debug!("Looking up install for {}", id);
    match lc {
        None => Err(eyre!("No config found")),
        Some(component_config) => match component_config.installs.get(&id) {
            None => Err(eyre!("No install found for id {id}")),
            // backwards compatible clause for when id's only pointed
            // to a directory and not more metadata
            Some(toml::Value::String(dir)) => {
                debug!("Found install in directory {}", dir);
                Ok(dir.to_owned())
            }
            Some(t @ toml::Value::Table(_)) => {
                if let Some(toml::Value::String(dir)) = t.get("dir") {
                    Ok(dir.to_string())
                } else {
                    Err(eyre!("No directory found for install id {id}"))
                }
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
        languages::Language::Elixir => Config {
            elixir: Some(new_lc),
            ..config
        },
    };

    write_config(config_file, new_config)
}

pub fn update_language_config(
    language: &languages::Language,
    id: &String,
    release: &String,
    dir: String,
    lc: LanguageConfig,
) -> Result<LanguageConfig> {
    let LanguageConfig {
        default: _,
        installs: mut table,
        default_build_options,
    } = lc;
    let mut id_table = toml::Table::new();
    id_table.insert("dir".to_string(), toml::Value::String(dir));
    id_table.insert(
        "release".to_string(),
        toml::Value::String(release.to_owned()),
    );

    if language == &languages::Language::Elixir {
        let otp_vsn = get_otp_major_vsn()?;
        id_table.insert("otp_vsn".to_string(), toml::Value::String(otp_vsn));
    }

    table.insert(id.clone(), toml::Value::Table(id_table));
    Ok(LanguageConfig {
        default: Some(id.to_owned()),
        installs: table.clone(),
        default_build_options: default_build_options.clone(),
    })
}

pub fn update_component_config(
    _kind: &components::Kind,
    id: &String,
    release: &String,
    dir: String,
    c: ComponentConfig,
) -> Result<ComponentConfig> {
    let ComponentConfig {
        default: _,
        installs: mut table,
        default_build_options,
    } = c;
    let mut id_table = toml::Table::new();
    id_table.insert("dir".to_string(), toml::Value::String(dir));
    id_table.insert(
        "release".to_string(),
        toml::Value::String(release.to_owned()),
    );

    table.insert(id.clone(), toml::Value::Table(id_table));
    Ok(ComponentConfig {
        default: Some(id.to_owned()),
        installs: table.clone(),
        default_build_options: default_build_options.clone(),
    })
}

pub fn add_install(
    language: &languages::Language,
    id: &String,
    release: &String,
    dir: String,
    config_file: String,
    config: Config,
) -> Result<()> {
    debug!("adding install {id} pointing to {dir}");
    let language_config = get_language_config(language, &config);

    let updated_language_config =
        update_language_config(language, id, release, dir, language_config.clone())?;

    let new_config = match language {
        languages::Language::Gleam => Config {
            gleam: Some(updated_language_config),
            ..config
        },
        languages::Language::Erlang => Config {
            erlang: Some(updated_language_config),
            ..config
        },
        languages::Language::Elixir => Config {
            elixir: Some(updated_language_config),
            ..config
        },
    };

    let _ = write_config(config_file, new_config);

    Ok(())
}

pub fn add_component_install(
    kind: &components::Kind,
    id: &String,
    release: &String,
    dir: String,
    config_file: String,
    config: Config,
) -> Result<()> {
    debug!("adding install {id} pointing to {dir}");
    let component_config = get_component_config(kind, &config);

    let updated_component_config =
        update_component_config(kind, id, release, dir, component_config.clone())?;

    let new_config = match kind {
        components::Kind::Elp => Config {
            elp: Some(updated_component_config),
            ..config
        },
        components::Kind::Rebar3 => Config {
            rebar3: Some(updated_component_config),
            ..config
        },
    };

    let _ = write_config(config_file, new_config);

    Ok(())
}

pub fn language_release_dir(
    language: &languages::Language,
    id: &String,
    force: bool,
) -> Result<PathBuf> {
    let data_dir = data_dir();
    let release_dir = data_dir
        .unwrap()
        .join("beamup")
        .join(language.to_string())
        .join(id);

    match release_dir.try_exists() {
        Ok(true) =>
            match force {
                true => {
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

pub fn data_dir() -> Result<PathBuf> {
    match dirs::data_local_dir() {
        Some(dir) => Ok(dir),
        None => Err(eyre!("No data directory available")),
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
            elixir: Some(LanguageConfig {
                default: None,
                installs: toml::Table::new(),
                default_build_options: None,
            }),
            elp: Some(ComponentConfig {
                default: None,
                installs: toml::Table::new(),
                default_build_options: None,
            }),
            rebar3: Some(ComponentConfig {
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
