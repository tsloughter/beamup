use crate::config;
use crate::github::GithubRepo;
use crate::languages;
use crate::languages::{Language, LanguageStruct};
use color_eyre::eyre::Result;

const LANGUAGE_STRING: &str = "gleam";

pub fn new(release: &str, id: &str, config: &config::Config) -> Result<LanguageStruct> {
    Ok(LanguageStruct {
        language: Language::Gleam,
        release_dir: languages::release_dir(LANGUAGE_STRING.to_string(), &id.to_string(), config)?,
        extract_dir: languages::release_dir(LANGUAGE_STRING.to_string(), &id.to_string(), config)?
            .join("bin"),
        asset_prefix: |release, _| asset_prefix(release),
        source_repo: get_source_github_repo(release, config),
        binary_repo: get_binary_github_repo(release, config),
        bins: bins(),
    })
}

#[cfg(unix)]
pub fn bins() -> Vec<(String, languages::Language)> {
    vec![("gleam".to_string(), languages::Language::Gleam)]
}

#[cfg(windows)]
pub fn bins() -> Vec<(String, languages::Language)> {
    vec![
        ("gleam.exe".to_string(), languages::Language::Gleam),
        ("gleam".to_string(), languages::Language::Gleam),
    ]
}

fn asset_prefix(release: &str) -> Result<String> {
    match (std::env::consts::ARCH, std::env::consts::OS) {
        ("x86_64", "linux") => Ok(format!("gleam-{release}-x86_64-unknown-linux-musl.tar.gz")),
        ("aarch64", "linux") => Ok(format!("gleam-{release}-aarch64-unknown-linux-musl.tar.gz")),
        ("x86_64", "macos") => Ok(format!("gleam-{release}-x86_64-apple-darwin.tar.gz")),
        ("aarch64", "macos") => Ok(format!("gleam-{release}-aarch64-apple-darwin.tar.gz")),
        ("x86_64", "windows") => Ok(format!("gleam-{release}-x86_64-pc-windows-msvc.zip")),
        _ => {
            // TODO: maybe turn this into an Option type and return None
            Ok("".to_string())
        }
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
        org: "gleam-lang".to_string(),
        repo: "gleam".to_string(),
    }
}
