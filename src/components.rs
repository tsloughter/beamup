use clap::ValueEnum;
pub mod elp;
pub mod rebar3;
use crate::config;
use crate::github::GithubRepo;
use crate::installs::Metadata;
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
    /// Heading used when listing installs.
    pub fn label(&self) -> &'static str {
        match self {
            Kind::Elp => "ELP",
            Kind::Rebar3 => "rebar3",
        }
    }

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

    fn install_metadata(&self, release: &str, config: &config::Config) -> Result<Metadata> {
        Ok(Metadata {
            release: release.to_owned(),
            otp_vsn: match self {
                // ELP assets are built per OTP major version; fall back to any release.
                Kind::Elp => config::get_otp_major_vsn(config).ok(),
                Kind::Rebar3 => None,
            },
            ..Metadata::default()
        })
    }

    fn asset_prefix(&self, _libc: &Option<Libc>, metadata: &Metadata) -> Result<regex::Regex> {
        match self {
            Kind::Elp => elp::asset_prefix(metadata.otp_vsn.as_deref().unwrap_or("")),
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
