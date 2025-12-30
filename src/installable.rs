use crate::config;
use crate::github::GithubRepo;
use clap::ValueEnum;
use color_eyre::eyre::Result;
use std::path::PathBuf;

#[derive(ValueEnum, Debug, Clone, PartialEq, EnumIter)]
pub enum Libc {
    Glibc,
    Musl,
}

pub trait Installable {
    fn default_build_options(&self, config: &config::Config) -> String;

    fn binary_repo(&self) -> GithubRepo;
    fn source_repo(&self) -> GithubRepo;

    fn release_dir(&self, id: &String) -> Result<PathBuf>;
    fn extract_dir(&self, id: &String) -> Result<PathBuf>;

    fn asset_prefix(&self, libc: &Option<Libc>) -> Result<regex::Regex>;
}
