use crate::{components, installs, languages};
use color_eyre::eyre::{eyre, Result, WrapErr};
use serde::{Deserialize, Serialize};
use std::{fs, path::*};

static LOCAL_CONFIG_FILE: &str = ".beamup.toml";
static CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub install_dir: PathBuf,
    erlang: Option<ToolConfig>,
    gleam: Option<ToolConfig>,
    elixir: Option<ToolConfig>,
    elp: Option<ToolConfig>,
    rebar3: Option<ToolConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ToolConfig {
    default: Option<String>,
    default_build_options: Option<String>,
}

impl Config {
    fn language(&self, language: &languages::Language) -> Option<&ToolConfig> {
        match language {
            languages::Language::Gleam => self.gleam.as_ref(),
            languages::Language::Erlang => self.erlang.as_ref(),
            languages::Language::Elixir => self.elixir.as_ref(),
        }
    }

    fn language_mut(&mut self, language: &languages::Language) -> &mut ToolConfig {
        match language {
            languages::Language::Gleam => &mut self.gleam,
            languages::Language::Erlang => &mut self.erlang,
            languages::Language::Elixir => &mut self.elixir,
        }
        .get_or_insert_with(ToolConfig::default)
    }

    fn component(&self, kind: &components::Kind) -> Option<&ToolConfig> {
        match kind {
            components::Kind::Elp => self.elp.as_ref(),
            components::Kind::Rebar3 => self.rebar3.as_ref(),
        }
    }
}

pub fn switch(language: &languages::Language, id: &str, config: &Config) -> Result<()> {
    installs::lookup(config, &language.to_string(), id)?;
    let mut local = local_config()?.unwrap_or_default();
    local.insert(language.to_string(), toml::Value::String(id.to_owned()));
    fs::write(LOCAL_CONFIG_FILE, toml::to_string(&local)?)?;
    Ok(())
}

fn selected_id(tool: &str, settings: Option<&ToolConfig>) -> Result<String> {
    if let Some(local) = local_config()? {
        if let Some(value) = local.get(tool) {
            return value.as_str().map(str::to_owned).ok_or_else(|| {
                eyre!("Expected an install ID string for {tool} in {LOCAL_CONFIG_FILE}")
            });
        }
    }
    settings
        .and_then(|s| s.default.clone())
        .ok_or_else(|| eyre!("No default found for {tool}"))
}

pub fn get_otp_major_vsn(config: &Config) -> Result<String> {
    let dir = install_to_use_by_language(&languages::Language::Erlang, config)
        .wrap_err("No default Erlang installation found. Install an Erlang version, like `beamup install erlang latest` or set a default with `beamup default erlang <ID>` first.")?;
    let releases_dir = dir.join("lib/erlang/releases");
    check_release_dir(&releases_dir).or_else(|_| check_release_dir(&dir.join("releases")))
}

fn check_release_dir(releases_dir: &Path) -> Result<String> {
    for entry in fs::read_dir(releases_dir)? {
        let entry = entry?;
        if entry.path().is_dir() {
            return entry
                .file_name()
                .into_string()
                .map_err(|name| eyre!("Unable to convert OTP version {name:?} to string"));
        }
    }
    Err(eyre!("No installed OTP release found in {releases_dir:?}"))
}

pub fn component_install_to_use(kind: &components::Kind, config: &Config) -> Result<PathBuf> {
    let name = kind.to_string();
    let id = selected_id(&name, config.component(kind))?;
    installs::lookup(config, &name, &id)
}

pub fn install_to_use_by_bin(bin: &str, config: &Config) -> Result<PathBuf> {
    let language = languages::bin_to_language(bin.to_owned(), config)?;
    install_to_use_by_language(&language, config)
}

fn install_to_use_by_language(language: &languages::Language, config: &Config) -> Result<PathBuf> {
    let name = language.to_string();
    let id = selected_id(&name, config.language(language))?;
    installs::lookup(config, &name, &id)
}

pub fn lookup_default_build_options(language: &languages::Language, config: &Config) -> String {
    config
        .language(language)
        .and_then(|c| c.default_build_options.clone())
        .unwrap_or_default()
}

pub fn set_default(
    language: &languages::Language,
    id: &str,
    config_file: String,
    mut config: Config,
) -> Result<()> {
    installs::lookup(&config, &language.to_string(), id)?;
    config.language_mut(language).default = Some(id.to_owned());
    write_config(config_file, config)
}

pub fn set_component_default(
    kind: &components::Kind,
    id: &str,
    config_file: String,
    mut config: Config,
) -> Result<()> {
    installs::lookup(&config, &kind.to_string(), id)?;
    let settings = match kind {
        components::Kind::Elp => &mut config.elp,
        components::Kind::Rebar3 => &mut config.rebar3,
    };
    settings.get_or_insert_with(ToolConfig::default).default = Some(id.to_owned());
    write_config(config_file, config)
}

pub fn bin_dir() -> PathBuf {
    dirs::executable_dir().unwrap_or_else(|| dirs::home_dir().unwrap().join(".beamup/bin"))
}

pub fn home_config_file() -> Result<String> {
    let config_dir = dirs::config_local_dir()
        .ok_or_else(|| eyre!("No config directory available"))?
        .join("beamup");
    let default_config = config_dir.join(CONFIG_FILE);
    fs::create_dir_all(&config_dir)?;
    if !default_config.exists() {
        let install_dir = dirs::data_local_dir()
            .ok_or_else(|| eyre!("No data directory available"))?
            .join("beamup");
        let config = Config {
            install_dir,
            erlang: None,
            gleam: None,
            elixir: None,
            elp: None,
            rebar3: None,
        };
        write_config(default_config.to_string_lossy().into_owned(), config)?;
        info!("Created a default config at {:?}", default_config);
    }
    Ok(default_config.to_string_lossy().into_owned())
}

fn local_config() -> Result<Option<toml::Table>> {
    match fs::read_to_string(LOCAL_CONFIG_FILE) {
        Ok(text) => Ok(Some(
            toml::from_str(&text).wrap_err("Failed to parse .beamup.toml")?,
        )),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn home_config() -> Result<(String, Config)> {
    let config_file = home_config_file()?;
    Ok((config_file.clone(), read_config(config_file)?))
}

pub fn read_config(file: String) -> Result<Config> {
    let text =
        fs::read_to_string(&file).wrap_err_with(|| format!("Failed to read config {file}"))?;
    let mut config: Config =
        toml::from_str(&text).wrap_err_with(|| format!("Failed to parse config {file}"))?;
    // Resolve relative roots against the config's directory, independent of the project's cwd.
    if config.install_dir.is_relative() {
        let config_file = std::path::absolute(file)?;
        config.install_dir = config_file.parent().unwrap().join(config.install_dir);
    }
    let mut normalized = PathBuf::new();
    for component in config.install_dir.components() {
        match component {
            Component::ParentDir => {
                normalized.pop();
            }
            Component::CurDir => {}
            other => normalized.push(other.as_os_str()),
        }
    }
    config.install_dir = normalized;
    Ok(config)
}

pub fn write_config(file: String, config: Config) -> Result<()> {
    fs::write(file, toml::to_string(&config)?)?;
    Ok(())
}
