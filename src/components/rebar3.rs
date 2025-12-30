use crate::components::Kind;
use crate::github::GithubRepo;
use color_eyre::eyre::{Result, WrapErr};
use regex::Regex;

const KIND_STRING: &str = "rebar3";

pub fn asset_prefix() -> Result<regex::Regex> {
    Regex::new(KIND_STRING).wrap_err("Unable to create asset regex")
}

pub fn get_github_repo() -> GithubRepo {
    GithubRepo {
        org: "erlang".to_string(),
        repo: KIND_STRING.to_string(),
    }
}

pub fn bins() -> Vec<(String, Kind)> {
    vec![(KIND_STRING.to_string(), Kind::Rebar3)]
}
