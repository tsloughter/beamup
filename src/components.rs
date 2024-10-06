use clap::ValueEnum;
pub mod elp;
pub mod rebar3;
pub mod erlang_ls;
use crate::config;
use crate::github::GithubRepo;
use color_eyre::eyre::Result;
use std::path::PathBuf;
use strum::IntoEnumIterator;

#[derive(ValueEnum, Debug, Clone, PartialEq, EnumIter)]
pub enum Kind {
    Elp,
    Rebar3,
    ErlangLS
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Kind::Elp => write!(f, "elp"),
            Kind::Rebar3 => write!(f, "rebar3"),
            Kind::ErlangLS => write!(f, "erlang_ls"),
        }
    }
}

pub fn print() {
    for l in Kind::iter() {
        println!("{:?}", l);
    }
}

pub fn bins() -> Vec<(String, Kind)> {
    let mut bins = vec![];
    let mut erlang_ls_bins = erlang_ls::bins();
    let mut elp_bins = elp::bins();
    let mut rebar3_bins = rebar3::bins();

    bins.append(&mut erlang_ls_bins);
    bins.append(&mut elp_bins);
    bins.append(&mut rebar3_bins);

    bins
}


pub struct Component {
    pub kind: Kind,
    pub release_dir: PathBuf,
    pub asset_prefix: String,
    pub repo: GithubRepo,
    pub bins: Vec<(String, Kind)>,
}

impl Component {
    pub fn new(kind: Kind, release: &str, config: &config::Config) -> Result<Self> {
        match kind {
            Kind::Elp => elp::new_component(release, config),
            Kind::Rebar3 => rebar3::new_component(release, config),
            Kind::ErlangLS => erlang_ls::new_component(release, config),
        }
    }
}

pub fn release_dir(kind_str: String, id: &String) -> Result<PathBuf> {
    let release_dir = config::data_dir()?.join("beamup").join(kind_str).join(id);

    Ok(release_dir)
}
