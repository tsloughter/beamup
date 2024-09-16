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

#[cfg(unix)]
pub const BIN_MAP: &[(&str, languages::Language)] = &[
    // Gleam
    ("gleam", languages::Language::Gleam),
    // Erlang
    ("ct_run", languages::Language::Erlang),
    ("dialyzer", languages::Language::Erlang),
    ("epmd", languages::Language::Erlang),
    ("erl", languages::Language::Erlang),
    ("erlc", languages::Language::Erlang),
    ("erl_call", languages::Language::Erlang),
    ("escript", languages::Language::Erlang),
    ("run_erl", languages::Language::Erlang),
    ("run_test", languages::Language::Erlang),
    ("to_erl", languages::Language::Erlang),
    ("typer", languages::Language::Erlang),
    // Elixir
    ("elixir", languages::Language::Elixir),
    ("elixirc", languages::Language::Elixir),
    ("iex", languages::Language::Elixir),
    ("mix", languages::Language::Elixir),
    ("mix", languages::Language::Elixir),
];

#[cfg(windows)]
pub const BIN_MAP: &[(&str, languages::Language)] = &[
    // Gleam
    ("gleam.exe", languages::Language::Gleam),
    // Erlang
    ("ct_run.exe", languages::Language::Erlang),
    ("dialyzer.exe", languages::Language::Erlang),
    ("epmd.exe", languages::Language::Erlang),
    ("erl.exe", languages::Language::Erlang),
    ("erlc.exe", languages::Language::Erlang),
    ("erl_call.exe", languages::Language::Erlang),
    ("escript.exe", languages::Language::Erlang),
    ("run_erl.exe", languages::Language::Erlang),
    ("run_test.exe", languages::Language::Erlang),
    ("to_erl.exe", languages::Language::Erlang),
    ("typer.exe", languages::Language::Erlang),
    // Elixir
    ("elixir.bat", languages::Language::Elixir),
    ("elixirc.bat", languages::Language::Elixir),
    ("iex.bat", languages::Language::Elixir),
    ("mix.bat", languages::Language::Elixir),
    ("mix.ps1", languages::Language::Elixir),
];

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

pub struct LanguageStruct {
    pub language: Language,
    pub release_dir: PathBuf,
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
        config: &config::Config,
    ) -> Result<Self> {
        match language {
            Language::Erlang => erlang::new(release, id, config),
            Language::Elixir => elixir::new(release, id, config),
            Language::Gleam => gleam::new(release, id, config),
        }
    }
}

pub fn bin_to_language(bin: &str) -> Result<&languages::Language> {
    match languages::BIN_MAP.iter().find(|&(k, _)| *k == bin) {
        Some((_, language)) => Ok(language),
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
