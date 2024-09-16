use crate::config;
use crate::github::GithubRepo;
use crate::languages;
use crate::languages::{Language, LanguageStruct};
use color_eyre::{eyre::eyre, eyre::Report, eyre::Result};

const LANGUAGE_STRING: &str = "gleam";

pub fn new(release: &str, id: &str, config: &config::Config) -> Result<LanguageStruct> {
    Ok(LanguageStruct {
        language: Language::Gleam,
        release_dir: languages::release_dir(LANGUAGE_STRING.to_string(), &id.to_string(), config)?,
        asset_prefix: asset_prefix(release, config)?,
        source_repo: get_source_github_repo(release, config),
        binary_repo: get_binary_github_repo(release, config),
        bins: bins(),
    })
}

#[cfg(unix)]
fn bins() -> Vec<(String, languages::Language)> {
    vec![
        ("elixir".to_string(), languages::Language::Elixir),
        ("elixirc".to_string(), languages::Language::Elixir),
        ("iex".to_string(), languages::Language::Elixir),
        ("mix".to_string(), languages::Language::Elixir),
    ]
}

#[cfg(windows)]
fn bins() -> Vec<(String, languages::Language)> {
    vec![
        ("elixir.bat".to_string(), languages::Language::Elixir),
        ("elixirc.bat".to_string(), languages::Language::Elixir),
        ("iex.bat".to_string(), languages::Language::Elixir),
        ("mix.bat".to_string(), languages::Language::Elixir),
        ("mix.ps1".to_string(), languages::Language::Elixir),
    ]
}

fn asset_prefix(release: &str, _config: &config::Config) -> Result<String> {
    let suffix = match (std::env::consts::ARCH, std::env::consts::OS) {
        ("x86_64", "linux") => "x86_64-unknown-linux-musl.tar.gz",
        ("aarch64", "linux") => "aarch64-unknown-linux-musl.tar.gz",
        ("x86_64", "macos") => "x86_64-apple-darwin.tar.gz",
        ("aarch64", "macos") => "aarch64-apple-darwin.tar.gz",
        ("x86_64", "windows") => "x86_64-pc-windows-msvc.zip",
        (arch, os) => {
            let e: Report = eyre!("no Gleam asset found to support arch:{arch} os:{os}");
            return Err(e);
        }
    };

    Ok(format!("gleam-{release}-{suffix}"))
}

pub fn get_binary_github_repo(_release: &str, _config: &config::Config) -> GithubRepo {
    get_github_repo()
}

pub fn get_source_github_repo(_release: &str, _config: &config::Config) -> GithubRepo {
    get_github_repo()
}

pub fn get_github_repo() -> GithubRepo {
    GithubRepo {
        org: "gleam-lang".to_string(),
        repo: "gleam".to_string(),
    }
}
