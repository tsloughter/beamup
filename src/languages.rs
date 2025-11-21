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

#[allow(dead_code)]
pub struct LanguageStruct {
    pub language: Language,
    pub release_dir: PathBuf,
    pub extract_dir: PathBuf,
    pub asset_prefix: String,
    pub source_repo: GithubRepo,
    pub binary_repo: GithubRepo,
    pub bins: Vec<(String, Language)>,
}

impl LanguageStruct {
    pub fn new(
        language: &Language,
        release: &str,
        id: &str,
        libc: &Option<Libc>,
        config: &config::Config,
    ) -> Result<Self> {
        match language {
            Language::Erlang => erlang::new(release, id, &libc, config),
            Language::Elixir => elixir::new(release, id, config),
            Language::Gleam => gleam::new(release, id, config),
        }
    }
}

pub fn bin_to_language(bin: String, config: &config::Config) -> Result<languages::Language> {
    match bins(config).iter().find(|&(k, _)| *k == bin) {
        Some((_, language)) => Ok(language.clone()),
        _ => Err(eyre!("No language to run command {bin} for found")),
    }
}

pub fn release_dir(language_str: String, id: &String, _config: &config::Config) -> Result<PathBuf> {
    let release_dir = config::data_dir()?
        .join("beamup")
        .join(language_str)
        .join(id);

    Ok(release_dir)
}
