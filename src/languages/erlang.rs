use crate::config;
use crate::github::GithubRepo;
use crate::languages;
use crate::languages::{Language, LanguageStruct, Libc};
use color_eyre::eyre::Result;

const LANGUAGE_STRING: &str = "erlang";

pub fn new(release: &str, id: &str, config: &config::Config) -> Result<LanguageStruct> {
    Ok(LanguageStruct {
        language: Language::Erlang,
        release_dir: languages::release_dir(LANGUAGE_STRING.to_string(), &id.to_string(), config)?,
        extract_dir: languages::release_dir(LANGUAGE_STRING.to_string(), &id.to_string(), config)?,
        asset_prefix: |release: &str, libc: &Option<Libc>| asset_prefix(release, libc),
        source_repo: get_source_github_repo(release, config),
        binary_repo: get_binary_github_repo(release, config),
        bins: bins(),
    })
}

#[cfg(unix)]
pub fn bins() -> Vec<(String, languages::Language)> {
    vec![
        ("ct_run".to_string(), languages::Language::Erlang),
        ("dialyzer".to_string(), languages::Language::Erlang),
        ("epmd".to_string(), languages::Language::Erlang),
        ("erl".to_string(), languages::Language::Erlang),
        ("erlc".to_string(), languages::Language::Erlang),
        ("erl_call".to_string(), languages::Language::Erlang),
        ("escript".to_string(), languages::Language::Erlang),
        ("run_erl".to_string(), languages::Language::Erlang),
        ("run_test".to_string(), languages::Language::Erlang),
        ("to_erl".to_string(), languages::Language::Erlang),
        ("typer".to_string(), languages::Language::Erlang),
    ]
}

#[cfg(windows)]
pub fn bins() -> Vec<(String, languages::Language)> {
    vec![
        ("ct_run.exe".to_string(), languages::Language::Erlang),
        ("dialyzer.exe".to_string(), languages::Language::Erlang),
        ("epmd.exe".to_string(), languages::Language::Erlang),
        ("erl.exe".to_string(), languages::Language::Erlang),
        ("erlc.exe".to_string(), languages::Language::Erlang),
        ("erl_call.exe".to_string(), languages::Language::Erlang),
        ("escript.exe".to_string(), languages::Language::Erlang),
        ("run_erl.exe".to_string(), languages::Language::Erlang),
        ("run_test.exe".to_string(), languages::Language::Erlang),
        ("to_erl.exe".to_string(), languages::Language::Erlang),
        ("typer.exe".to_string(), languages::Language::Erlang),
        ("ct_run".to_string(), languages::Language::Erlang),
        ("dialyzer".to_string(), languages::Language::Erlang),
        ("epmd".to_string(), languages::Language::Erlang),
        ("erl".to_string(), languages::Language::Erlang),
        ("erlc".to_string(), languages::Language::Erlang),
        ("erl_call".to_string(), languages::Language::Erlang),
        ("escript".to_string(), languages::Language::Erlang),
        ("run_erl".to_string(), languages::Language::Erlang),
        ("run_test".to_string(), languages::Language::Erlang),
        ("to_erl".to_string(), languages::Language::Erlang),
        ("typer".to_string(), languages::Language::Erlang),
    ]
}

fn asset_prefix(release: &str, libc: &Option<Libc>) -> Result<String> {
    let vsn = release.strip_prefix("OTP-").unwrap_or(release);
    let libc = match libc {
        None => "",
        Some(Libc::Glibc) => "-glibc",
        Some(Libc::Musl) => "-musl",
    };
    match (std::env::consts::ARCH, std::env::consts::OS) {
        ("x86", "windows") => Ok(format!("otp_win32_{vsn}.exe")),
        ("x86_64", "windows") => Ok(format!("otp_win64_{vsn}.exe")),
        ("x86_64", "macos") => Ok(format!("otp-x86_64-apple-darwin.tar.gz")),
        ("aarch64", "macos") => Ok(format!("otp-aarch64-apple-darwin.tar.gz")),
        ("aarch64", "linux") => Ok(format!("erlang-{vsn}-arm64{libc}.tar.gz")),
        ("x86_64", "linux") => Ok(format!("erlang-{vsn}-x64{libc}.tar.gz")),
        _ => {
            // TODO: maybe turn this into an Option type and return None
            Ok("".to_string())
        }
    }
}

pub fn get_binary_github_repo(_release: &str, _config: &config::Config) -> GithubRepo {
    match (std::env::consts::ARCH, std::env::consts::OS) {
        (_, "windows") => GithubRepo {
            org: "erlang".to_string(),
            repo: "otp".to_string(),
        },
        (_, "macos") => GithubRepo {
            org: "erlef".to_string(),
            repo: "otp_builds".to_string(),
        },
        (_, _) => GithubRepo {
            org: "gleam-community".to_string(),
            repo: "erlang-linux-builds".to_string(),
        },
    }
}

pub fn get_source_github_repo(_release: &str, _config: &config::Config) -> GithubRepo {
    GithubRepo {
        org: "erlang".to_string(),
        repo: "otp".to_string(),
    }
}
