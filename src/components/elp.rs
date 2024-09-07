use crate::components::{release_dir, Component, Kind};
use crate::github::GithubRepo;
use color_eyre::{eyre::eyre, eyre::Report, eyre::Result};

static KIND_STRING: &'static str = "elp";

pub fn new_component(release: &str) -> Result<Component> {
    Ok(Component {
        kind: Kind::Elp,
        release_dir: release_dir(KIND_STRING.to_string(), &release.to_string())?,
        asset_prefix: asset_prefix(&release.to_string())?,
        repo: get_github_repo(),
        bins: bins(),
    })
}

fn bins() -> Vec<(String, Kind)> {
    vec![(KIND_STRING.to_string(), Kind::Elp)]
}

fn asset_prefix(_release: &str) -> Result<String> {
    let suffix = match (std::env::consts::ARCH, std::env::consts::OS) {
        ("x86_64", "linux") => "linux-x86_64-unknown-linux-gnu",
        ("aarch64", "linux") => "linux-aarch64-unknown-linux-gnu",
        ("x86_64", "macos") => "macos-x86_64-apple-darwin",
        ("aarch64", "macos") => "macos-aarch64-apple-darwin",
        (arch, os) => {
            let e: Report = eyre!("no elp asset found to support arch:{arch} os:{os}");
            return Err(e);
        }
    };

    Ok(format!("elp-{suffix}"))
}

fn get_github_repo() -> GithubRepo {
    GithubRepo {
        org: "WhatsApp".to_string(),
        repo: "erlang-language-platform".to_string(),
    }
}
