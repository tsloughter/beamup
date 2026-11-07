use clap::ValueEnum;
pub mod elp;
pub mod rebar3;
use crate::config;
use crate::github::GithubRepo;
use color_eyre::eyre::Result;
use std::path::{Path, PathBuf};
use strum::IntoEnumIterator;

use crate::installable::{Installable, Libc};

#[derive(ValueEnum, Debug, Clone, PartialEq, EnumIter)]
pub enum Kind {
    Elp,
    Rebar3,
}

impl Kind {
    pub fn bins(&self) -> Vec<(String, Kind)> {
        match self {
            Kind::Elp => elp::bins(),
            Kind::Rebar3 => rebar3::bins(),
        }
    }
}

impl Installable for Kind {
    fn binary_repo(&self) -> GithubRepo {
        match self {
            Kind::Elp => elp::get_github_repo(),
            Kind::Rebar3 => rebar3::get_github_repo(),
        }
    }

    fn extract_dir(&self, release_dir: &Path) -> PathBuf {
        release_dir.join("bin")
    }

    fn asset_prefix(&self, _libc: &Option<Libc>, config: &config::Config) -> Result<regex::Regex> {
        match self {
            Kind::Elp => elp::asset_prefix(config),
            Kind::Rebar3 => rebar3::asset_prefix(),
        }
    }
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Kind::Elp => write!(f, "elp"),
            Kind::Rebar3 => write!(f, "rebar3"),
        }
    }
}

pub fn print() {
    for l in Kind::iter() {
        println!("{:?}", l);
    }
}

pub fn bins() -> Vec<(String, Kind)> {
    Kind::iter().flat_map(|kind| kind.bins()).collect()
}
