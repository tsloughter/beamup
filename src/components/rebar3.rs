use crate::config;
use crate::components::{release_dir, Component, Kind};
use crate::github::GithubRepo;
use color_eyre::eyre::Result;

const KIND_STRING: &str = "rebar3";

pub fn new_component(release: &str, config: &config::Config) -> Result<Component> {
    Ok(Component {
        kind: Kind::Rebar3,
        release_dir: release_dir(KIND_STRING.to_string(), &release.to_string())?,
        asset_prefix: asset_prefix(release, config)?,
        repo: get_github_repo(),
        bins: bins(),
    })
}

fn asset_prefix(_release: &str, _config: &config::Config) -> Result<String> {
    Ok(KIND_STRING.to_string())
}

fn get_github_repo() -> GithubRepo {
    GithubRepo {
        org: "erlang".to_string(),
        repo: KIND_STRING.to_string(),
    }
}

pub fn bins() -> Vec<(String, Kind)> {
    vec![(KIND_STRING.to_string(), Kind::Rebar3)]
}
