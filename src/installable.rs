use crate::config;
use crate::github::GithubRepo;
use crate::installs::Metadata;
use clap::ValueEnum;
use color_eyre::eyre::Result;
use std::path::{Path, PathBuf};

#[derive(ValueEnum, Debug, Clone, PartialEq, EnumIter)]
pub enum Libc {
    Glibc,
    Musl,
}

pub trait Installable: std::fmt::Display {
    fn binary_repo(&self) -> GithubRepo;
    fn source_repo(&self) -> GithubRepo {
        self.binary_repo()
    }

    fn release_dir(&self, id: &str, config: &config::Config) -> Result<PathBuf> {
        crate::installs::path(config, &self.to_string(), id)
    }
    fn extract_dir(&self, release_dir: &Path) -> PathBuf;

    /// Metadata recorded alongside the install. Anything asset selection
    /// depends on, such as the OTP major version, is resolved once here.
    fn install_metadata(&self, release: &str, _config: &config::Config) -> Result<Metadata> {
        Ok(Metadata {
            release: release.to_owned(),
            ..Metadata::default()
        })
    }

    fn asset_prefix(&self, libc: &Option<Libc>, metadata: &Metadata) -> Result<regex::Regex>;
}

pub trait Buildable: Installable {
    fn default_build_options(&self, config: &config::Config) -> String;
}
