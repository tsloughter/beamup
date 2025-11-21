use crate::config;
use crate::github::GithubRepo;
use crate::languages;
use crate::languages::{Language, LanguageStruct};
use color_eyre::eyre::Result;

const LANGUAGE_STRING: &str = "elixir";

pub fn new(release: &str, id: &str, config: &config::Config) -> Result<LanguageStruct> {
    Ok(LanguageStruct {
        language: Language::Elixir,
        release_dir: languages::release_dir(LANGUAGE_STRING.to_string(), &id.to_string(), config)?,
        extract_dir: languages::release_dir(LANGUAGE_STRING.to_string(), &id.to_string(), config)?,
        // TODO: make this into a closure or method so we only have to run it when we need the asset
        asset_prefix: asset_prefix(release, config)?,
        source_repo: get_source_github_repo(release, config),
        binary_repo: get_binary_github_repo(release, config),
        bins: bins(),
    })
}

#[cfg(unix)]
pub fn bins() -> Vec<(String, languages::Language)> {
    vec![
        ("elixir".to_string(), languages::Language::Elixir),
        ("elixirc".to_string(), languages::Language::Elixir),
        ("iex".to_string(), languages::Language::Elixir),
        ("mix".to_string(), languages::Language::Elixir),
    ]
}

#[cfg(windows)]
pub fn bins() -> Vec<(String, languages::Language)> {
    vec![
        ("elixir.bat".to_string(), languages::Language::Elixir),
        ("elixirc.bat".to_string(), languages::Language::Elixir),
        ("iex.bat".to_string(), languages::Language::Elixir),
        ("mix.bat".to_string(), languages::Language::Elixir),
        ("mix.ps1".to_string(), languages::Language::Elixir),
    ]
}

fn asset_prefix(_release: &str, _config: &config::Config) -> Result<String> {
    // find dir of active Erlang
    match config::get_otp_major_vsn() {
        Ok(otp_major_vsn) => Ok(format!("elixir-otp-{otp_major_vsn:}.zip")),
        // see the above TODO for why we return an empty string here
        Err(_) => Ok("".to_string()),
    }
}

pub fn get_binary_github_repo(_release: &str, _config: &config::Config) -> GithubRepo {
    get_github_repo()
}

pub fn get_source_github_repo(_release: &str, _config: &config::Config) -> GithubRepo {
    get_github_repo()
}

pub fn get_github_repo() -> GithubRepo {
    GithubRepo {
        org: "elixir-lang".to_string(),
        repo: "elixir".to_string(),
    }
}
