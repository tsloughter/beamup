use crate::components::{release_dir, Component, Kind};
use crate::config;
use crate::github::GithubRepo;
use color_eyre::eyre::{eyre, Result, WrapErr};
use regex::Regex;

const KIND_STRING: &str = "elp";

pub fn new_component(release: &str) -> Result<Component> {
    Ok(Component {
        kind: Kind::Elp,
        release_dir: release_dir(KIND_STRING.to_string(), &release.to_string())?,
        asset_prefix: asset_prefix()?,
        repo: get_github_repo(),
        bins: bins(),
    })
}

fn bins() -> Vec<(String, Kind)> {
    vec![(KIND_STRING.to_string(), Kind::Elp)]
}

fn asset_prefix() -> Result<regex::Regex> {
    // find dir of active Erlang
    match config::get_otp_major_vsn() {
        Ok(otp_major_vsn) => match (std::env::consts::ARCH, std::env::consts::OS) {
            ("x86_64", "linux") => Regex::new(
                format!("elp-linux-x86_64-unknown-linux-gnu-otp-{otp_major_vsn:}").as_str(),
            )
            .wrap_err("Failed to create asset regex"),
            ("aarch64", "linux") => Regex::new(
                format!("elp-linux-aarch64-unknown-linux-gnu-otp-{otp_major_vsn:}").as_str(),
            )
            .wrap_err("Failed to create asset regex"),
            ("x86_64", "macos") => {
                Regex::new(format!("elp-macos-x86_64-apple-darwin-otp-{otp_major_vsn:}").as_str())
                    .wrap_err("Failed to create asset regex")
            }
            ("aarch64", "macos") => {
                Regex::new(format!("elp-macos-aarch64-apple-darwin-otp-{otp_major_vsn:}").as_str())
                    .wrap_err("Failed to create asset regex")
            }
            _ => {
                // TODO: maybe turn this into an Option type and return None
                Regex::new("").wrap_err("Failed to create asset regex")
            }
        },
        Err(_) => Err(eyre!("No Erlang install found.")),
    }
}

fn get_github_repo() -> GithubRepo {
    GithubRepo {
        org: "WhatsApp".to_string(),
        repo: "erlang-language-platform".to_string(),
    }
}
