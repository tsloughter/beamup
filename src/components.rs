use clap::ValueEnum;
pub mod elp;
pub mod rebar3;
use crate::config;
use crate::github::GithubRepo;
use color_eyre::eyre::Result;
use std::path::PathBuf;
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
    fn default_build_options(&self, _config: &config::Config) -> String {
        "".to_string()
    }

    fn binary_repo(&self) -> GithubRepo {
        match self {
            Kind::Elp => elp::get_github_repo(),
            Kind::Rebar3 => rebar3::get_github_repo(),
        }
    }

    fn source_repo(&self) -> GithubRepo {
        self.binary_repo()
    }

    fn release_dir(&self, id: &String) -> Result<PathBuf> {
        release_dir(self.to_string(), id)
    }

    fn extract_dir(&self, id: &String) -> Result<PathBuf> {
        self.release_dir(id)
    }

    fn asset_prefix(&self, _libc: &Option<Libc>) -> Result<regex::Regex> {
        match self {
            Kind::Elp => elp::asset_prefix(),
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

pub fn release_dir(kind_str: String, id: &String) -> Result<PathBuf> {
    let release_dir = config::data_dir()?.join("beamup").join(kind_str).join(id);

    Ok(release_dir)
}
