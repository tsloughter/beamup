use crate::config;
use crate::github::GithubRepo;
use crate::languages;
use clap::ValueEnum;
use color_eyre::{eyre::eyre, eyre::Result};
use std::path::PathBuf;
use strum::IntoEnumIterator;
pub mod elixir;
pub mod erlang;
pub mod gleam;

#[derive(ValueEnum, Debug, Clone, PartialEq, EnumIter)]
pub enum Libc {
    Glibc,
    Musl,
}

#[derive(ValueEnum, Debug, Clone, PartialEq, EnumIter)]
pub enum Language {
    Elixir,
    Erlang,
    Gleam,
}

pub fn print() {
    for l in Language::iter() {
        println!("{:?}", l);
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Language::Erlang => write!(f, "erlang"),
            Language::Gleam => write!(f, "gleam"),
            Language::Elixir => write!(f, "elixir"),
        }
    }
}

pub fn bins(_config: &config::Config) -> Vec<(String, Language)> {
    let mut bins = vec![];
    let mut elixir_bins = elixir::bins();
    let mut erlang_bins = erlang::bins();
    let mut gleam_bins = gleam::bins();

    bins.append(&mut elixir_bins);
    bins.append(&mut erlang_bins);
    bins.append(&mut gleam_bins);

    bins
}

pub struct Elixir;
pub struct Erlang;
pub struct Gleam;

pub trait Installable {
    fn language(&self) -> Language;

    fn bins(&self) -> Vec<(String, Language)>;

    fn binary_repo(&self) -> GithubRepo;
    fn source_repo(&self) -> GithubRepo;

    fn release_dir(&self, id: &str) -> Result<PathBuf>;
    fn extract_dir(&self, id: &str) -> Result<PathBuf>;

    fn asset_prefix(&self, release: &str, libc: &Option<Libc>) -> Result<String>;
}

impl Installable for Elixir {
    fn language(&self) -> Language {
        Language::Elixir
    }

    fn bins(&self) -> Vec<(String, Language)> {
        elixir::bins()
    }

    fn binary_repo(&self) -> GithubRepo {
        GithubRepo {
            org: "elixir-lang".to_string(),
            repo: "elixir".to_string(),
        }
    }

    fn source_repo(&self) -> GithubRepo {
        GithubRepo {
            org: "elixir-lang".to_string(),
            repo: "elixir".to_string(),
        }
    }

    fn release_dir(&self, id: &str) -> Result<PathBuf> {
        languages::release_dir("elixir", id)
    }

    fn extract_dir(&self, id: &str) -> Result<PathBuf> {
        languages::release_dir("elixir", id)
    }

    fn asset_prefix(&self, release: &str, _libc: &Option<Libc>) -> Result<String> {
        elixir::asset_prefix(release)
    }
}

impl Installable for Erlang {
    fn language(&self) -> Language {
        Language::Erlang
    }

    fn bins(&self) -> Vec<(String, Language)> {
        erlang::bins()
    }

    fn source_repo(&self) -> GithubRepo {
        GithubRepo {
            org: "erlang".to_string(),
            repo: "otp".to_string(),
        }
    }

    fn binary_repo(&self) -> GithubRepo {
        match std::env::consts::OS {
            "windows" => GithubRepo {
                org: "erlang".to_string(),
                repo: "otp".to_string(),
            },
            "macos" => GithubRepo {
                org: "erlef".to_string(),
                repo: "otp_builds".to_string(),
            },
            _ => GithubRepo {
                org: "gleam-community".to_string(),
                repo: "erlang-linux-builds".to_string(),
            },
        }
    }

    fn release_dir(&self, id: &str) -> Result<PathBuf> {
        languages::release_dir("erlang", id)
    }

    fn extract_dir(&self, id: &str) -> Result<PathBuf> {
        languages::release_dir("erlang", id)
    }

    fn asset_prefix(&self, release: &str, libc: &Option<Libc>) -> Result<String> {
        erlang::asset_prefix(release, libc)
    }
}

impl Installable for Gleam {
    fn language(&self) -> Language {
        Language::Gleam
    }

    fn bins(&self) -> Vec<(String, Language)> {
        gleam::bins()
    }

    fn binary_repo(&self) -> GithubRepo {
        self.get_github_repo()
    }

    fn source_repo(&self) -> GithubRepo {
        self.get_github_repo()
    }

    fn release_dir(&self, id: &str) -> Result<PathBuf> {
        languages::release_dir("gleam", id)
    }

    fn extract_dir(&self, id: &str) -> Result<PathBuf> {
        languages::release_dir("gleam", id)
    }

    fn asset_prefix(&self, release: &str, _libc: &Option<Libc>) -> Result<String> {
        gleam::asset_prefix(release)
    }
}

impl Gleam {
    fn get_github_repo(&self) -> GithubRepo {
        GithubRepo {
            org: "gleam-lang".to_string(),
            repo: "gleam".to_string(),
        }
    }
}

pub fn bin_to_language(bin: String, config: &config::Config) -> Result<languages::Language> {
    match bins(config).iter().find(|&(k, _)| *k == bin) {
        Some((_, language)) => Ok(language.clone()),
        _ => Err(eyre!("No language to run command {bin} for found")),
    }
}

pub fn release_dir(language_str: &str, id: &str) -> Result<PathBuf> {
    let release_dir = config::data_dir()?
        .join("beamup")
        .join(language_str)
        .join(id);

    Ok(release_dir)
}
