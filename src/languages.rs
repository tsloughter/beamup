use crate::config;
use crate::github::GithubRepo;
use crate::languages;
use clap::ValueEnum;
use color_eyre::eyre::{eyre, Result};
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

pub trait Installable {
    fn default_build_options(&self, config: &config::Config) -> String;

    fn binary_repo(&self) -> GithubRepo;
    fn source_repo(&self) -> GithubRepo;

    fn release_dir(&self, id: &String) -> Result<PathBuf>;
    fn extract_dir(&self, id: &String) -> Result<PathBuf>;

    fn asset_prefix(&self, release: &str, libc: &Option<Libc>) -> Result<regex::Regex>;
}

impl Installable for Language {
    fn default_build_options(&self, config: &config::Config) -> String {
        config::lookup_default_build_options(self, config)
    }

    fn binary_repo(&self) -> GithubRepo {
        match self {
            Language::Elixir => GithubRepo {
                org: "elixir-lang".to_string(),
                repo: "elixir".to_string(),
            },
            Language::Erlang => match std::env::consts::OS {
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
            },
            Language::Gleam => GithubRepo {
                org: "gleam-lang".to_string(),
                repo: "gleam".to_string(),
            },
        }
    }

    fn source_repo(&self) -> GithubRepo {
        match self {
            Language::Elixir => GithubRepo {
                org: "elixir-lang".to_string(),
                repo: "elixir".to_string(),
            },
            Language::Erlang => GithubRepo {
                org: "erlang".to_string(),
                repo: "otp".to_string(),
            },
            Language::Gleam => GithubRepo {
                org: "gleam-lang".to_string(),
                repo: "gleam".to_string(),
            },
        }
    }

    fn release_dir(&self, id: &String) -> Result<PathBuf> {
        languages::release_dir(self.to_string(), id)
    }

    fn extract_dir(&self, id: &String) -> Result<PathBuf> {
        languages::release_dir(self.to_string(), id)
    }

    fn asset_prefix(&self, release: &str, libc: &Option<Libc>) -> Result<regex::Regex> {
        match self {
            Language::Elixir => elixir::asset_prefix(release),
            Language::Erlang => erlang::asset_prefix(release, libc),
            Language::Gleam => gleam::asset_prefix(release),
        }
    }
}

pub fn bin_to_language(bin: String, config: &config::Config) -> Result<languages::Language> {
    match bins(config).iter().find(|&(k, _)| *k == bin) {
        Some((_, language)) => Ok(language.clone()),
        _ => Err(eyre!("No language to run command {bin} for found")),
    }
}

pub fn release_dir(language_str: String, id: &String) -> Result<PathBuf> {
    let release_dir = config::data_dir()?
        .join("beamup")
        .join(language_str)
        .join(id);

    Ok(release_dir)
}
