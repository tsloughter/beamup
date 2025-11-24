use crate::languages::Language;
use color_eyre::eyre::{eyre, Result, WrapErr};
use regex::Regex;

#[cfg(unix)]
pub fn bins() -> Vec<(String, Language)> {
    vec![("gleam".to_string(), Language::Gleam)]
}

#[cfg(windows)]
pub fn bins() -> Vec<(String, Language)> {
    vec![
        ("gleam.exe".to_string(), Language::Gleam),
        ("gleam".to_string(), Language::Gleam),
    ]
}

pub fn asset_prefix() -> Result<regex::Regex> {
    match (std::env::consts::ARCH, std::env::consts::OS) {
        ("x86_64", "linux") => Regex::new("gleam-.*-x86_64-unknown-linux-musl.tar.gz")
            .wrap_err("Unable to create asset regex"),
        ("aarch64", "linux") => Regex::new("gleam-.*-aarch64-unknown-linux-musl.tar.gz")
            .wrap_err("Unable to create asset regex"),
        ("x86_64", "macos") => Regex::new("gleam-.*-x86_64-apple-darwin.tar.gz")
            .wrap_err("Unable to create asset regex"),
        ("aarch64", "macos") => Regex::new("gleam-.*-aarch64-apple-darwin.tar.gz")
            .wrap_err("Unable to create asset regex"),
        ("x86_64", "windows") => Regex::new("gleam-.*-x86_64-pc-windows-msvc.zip")
            .wrap_err("Unable to create asset regex"),
        _ => Err(eyre!("Unknown architecture or OS for installing gleam")),
    }
}
