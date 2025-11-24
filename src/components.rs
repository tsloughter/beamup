use clap::ValueEnum;
pub mod elp;
pub mod rebar3;
use crate::config;
use crate::github::GithubRepo;
use color_eyre::eyre::Result;
use std::path::PathBuf;
use strum::IntoEnumIterator;

#[derive(ValueEnum, Debug, Clone, PartialEq, EnumIter)]
pub enum Kind {
    Elp,
    Rebar3,
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
    Kind::iter()
        .flat_map(|kind| {
            // TODO: Fix me, needs to be able to fail properly if data_local_dir fails
            let c = Component::new(kind.clone(), "").unwrap();

            c.bins
        })
        .collect()
}

pub struct Component {
    pub kind: Kind,
    pub release_dir: PathBuf,
    pub asset_prefix: regex::Regex,
    pub repo: GithubRepo,
    pub bins: Vec<(String, Kind)>,
}

impl Component {
    pub fn new(kind: Kind, release: &str) -> Result<Self> {
        match kind {
            Kind::Elp => elp::new_component(release),
            Kind::Rebar3 => rebar3::new_component(release),
        }
    }
}

pub fn release_dir(kind_str: String, id: &String) -> Result<PathBuf> {
    let release_dir = config::data_dir()?.join("beamup").join(kind_str).join(id);

    Ok(release_dir)
}
