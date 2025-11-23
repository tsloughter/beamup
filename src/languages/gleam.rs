use crate::languages::Language;
use color_eyre::eyre::Result;

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

pub fn asset_prefix(release: &str) -> Result<String> {
    match (std::env::consts::ARCH, std::env::consts::OS) {
        ("x86_64", "linux") => Ok(format!("gleam-{release}-x86_64-unknown-linux-musl.tar.gz")),
        ("aarch64", "linux") => Ok(format!("gleam-{release}-aarch64-unknown-linux-musl.tar.gz")),
        ("x86_64", "macos") => Ok(format!("gleam-{release}-x86_64-apple-darwin.tar.gz")),
        ("aarch64", "macos") => Ok(format!("gleam-{release}-aarch64-apple-darwin.tar.gz")),
        ("x86_64", "windows") => Ok(format!("gleam-{release}-x86_64-pc-windows-msvc.zip")),
        _ => {
            // TODO: maybe turn this into an Option type and return None
            Ok("".to_string())
        }
    }
}
