use crate::components::{release_dir, Component, Kind};
use crate::github::GithubRepo;
use color_eyre::eyre::{Result, WrapErr};
use regex::Regex;

const KIND_STRING: &str = "rebar3";

pub fn new_component(release: &str) -> Result<Component> {
    Ok(Component {
        kind: Kind::Rebar3,
        release_dir: release_dir(KIND_STRING.to_string(), &release.to_string())?,
        asset_prefix: asset_prefix(release)?,
        repo: get_github_repo(),
        bins: bins(),
    })
}

fn asset_prefix(_release: &str) -> Result<regex::Regex> {
    Regex::new(KIND_STRING).wrap_err("Unable to create asset regex")
}

fn get_github_repo() -> GithubRepo {
    GithubRepo {
        org: "erlang".to_string(),
        repo: KIND_STRING.to_string(),
    }
}

fn bins() -> Vec<(String, Kind)> {
    vec![(KIND_STRING.to_string(), Kind::Rebar3)]
}
