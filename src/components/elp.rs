use crate::components::{release_dir, Component, Kind};
use crate::github::GithubRepo;
use color_eyre::eyre::Result;

const KIND_STRING: &str = "elp";

pub fn new_component(release: &str) -> Result<Component> {
    Ok(Component {
        kind: Kind::Elp,
        release_dir: release_dir(KIND_STRING.to_string(), &release.to_string())?,
        asset_prefix: asset_prefix(release)?,
        repo: get_github_repo(),
        bins: bins(),
    })
}

fn bins() -> Vec<(String, Kind)> {
    vec![(KIND_STRING.to_string(), Kind::Elp)]
}

fn asset_prefix(_release: &str) -> Result<String> {
    match (std::env::consts::ARCH, std::env::consts::OS) {
        ("x86_64", "linux") => Ok("elp-linux-x86_64-unknown-linux-gnu".to_string()),
        ("aarch64", "linux") => Ok("elp-linux-aarch64-unknown-linux-gnu".to_string()),
        ("x86_64", "macos") => Ok("elp-macos-x86_64-apple-darwin".to_string()),
        ("aarch64", "macos") => Ok("elp-macos-aarch64-apple-darwin".to_string()),
        _ => {
            // TODO: maybe turn this into an Option type and return None
            Ok("".to_string())
        }
    }
}

fn get_github_repo() -> GithubRepo {
    GithubRepo {
        org: "WhatsApp".to_string(),
        repo: "erlang-language-platform".to_string(),
    }
}
