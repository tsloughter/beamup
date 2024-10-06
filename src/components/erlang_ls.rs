use crate::config;
use crate::components::{release_dir, Component, Kind};
use crate::github::GithubRepo;
use color_eyre::eyre::Result;

const KIND_STRING: &str = "erlang_ls";

pub fn new_component(release: &str, config: &config::Config) -> Result<Component> {
    Ok(Component {
        kind: Kind::ErlangLS,
        release_dir: release_dir(KIND_STRING.to_string(), &release.to_string())?,
        asset_prefix: asset_prefix(release, config)?,
        repo: get_github_repo(),
        bins: bins(),
    })
}

pub fn bins() -> Vec<(String, Kind)> {
    vec![(KIND_STRING.to_string(), Kind::ErlangLS)]
}

fn asset_prefix(_release: &str, _config: &config::Config) -> Result<String> {
    let otp_major_vsn = config::get_otp_major_vsn()?;
    match (std::env::consts::ARCH, std::env::consts::OS) {
        (_, "linux") => Ok(format!("erlang_ls-linux-{otp_major_vsn:}")),
        (_, "macos") => Ok(format!("erlang_ls-{otp_major_vsn:}-macos")),
        (_, "windows") => Ok(format!("erlang_ls-win32")),
        _ => {
            // TODO: maybe turn this into an Option type and return None
            Ok("".to_string())
        }
    }
}

fn get_github_repo() -> GithubRepo {
    GithubRepo {
        org: "erlang-ls".to_string(),
        repo: "erlang_ls".to_string(),
    }
}
