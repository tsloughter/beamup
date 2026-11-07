use crate::config;
use crate::github::GithubRepo;
use clap::ValueEnum;
use color_eyre::eyre::{eyre, Result};
use std::path::{Path, PathBuf};
use strum::IntoEnumIterator;
pub mod elixir;
pub mod erlang;
pub mod gleam;

use crate::installable::{Buildable, Installable, Libc};

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

impl Buildable for Language {
    fn default_build_options(&self, config: &config::Config) -> String {
        config::lookup_default_build_options(self, config)
    }
}

impl Installable for Language {
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

    fn extract_dir(&self, release_dir: &Path) -> PathBuf {
        match self {
            Language::Gleam => release_dir.join("bin"),
            _ => release_dir.to_owned(),
        }
    }

    fn asset_prefix(&self, libc: &Option<Libc>, config: &config::Config) -> Result<regex::Regex> {
        match self {
            Language::Elixir => elixir::asset_prefix(config),
            Language::Erlang => erlang::asset_prefix(libc),
            Language::Gleam => gleam::asset_prefix(),
        }
    }
}

pub fn bin_to_language(bin: String, config: &config::Config) -> Result<Language> {
    match bins(config).iter().find(|&(k, _)| *k == bin) {
        Some((_, language)) => Ok(language.clone()),
        _ => Err(eyre!("No language to run command {bin} for found")),
    }
}
