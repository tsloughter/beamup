use crate::languages::{Language, Libc};
use color_eyre::eyre::{eyre, Result, WrapErr};
use regex::Regex;

#[cfg(unix)]
pub fn bins() -> Vec<(String, Language)> {
    vec![
        ("ct_run".to_string(), Language::Erlang),
        ("dialyzer".to_string(), Language::Erlang),
        ("epmd".to_string(), Language::Erlang),
        ("erl".to_string(), Language::Erlang),
        ("erlc".to_string(), Language::Erlang),
        ("erl_call".to_string(), Language::Erlang),
        ("escript".to_string(), Language::Erlang),
        ("run_erl".to_string(), Language::Erlang),
        ("run_test".to_string(), Language::Erlang),
        ("to_erl".to_string(), Language::Erlang),
        ("typer".to_string(), Language::Erlang),
    ]
}

#[cfg(windows)]
pub fn bins() -> Vec<(String, Language)> {
    vec![
        ("ct_run.exe".to_string(), Language::Erlang),
        ("dialyzer.exe".to_string(), Language::Erlang),
        ("epmd.exe".to_string(), Language::Erlang),
        ("erl.exe".to_string(), Language::Erlang),
        ("erlc.exe".to_string(), Language::Erlang),
        ("erl_call.exe".to_string(), Language::Erlang),
        ("escript.exe".to_string(), Language::Erlang),
        ("run_erl.exe".to_string(), Language::Erlang),
        ("run_test.exe".to_string(), Language::Erlang),
        ("to_erl.exe".to_string(), Language::Erlang),
        ("typer.exe".to_string(), Language::Erlang),
        ("ct_run".to_string(), Language::Erlang),
        ("dialyzer".to_string(), Language::Erlang),
        ("epmd".to_string(), Language::Erlang),
        ("erl".to_string(), Language::Erlang),
        ("erlc".to_string(), Language::Erlang),
        ("erl_call".to_string(), Language::Erlang),
        ("escript".to_string(), Language::Erlang),
        ("run_erl".to_string(), Language::Erlang),
        ("run_test".to_string(), Language::Erlang),
        ("to_erl".to_string(), Language::Erlang),
        ("typer".to_string(), Language::Erlang),
    ]
}

pub fn asset_prefix(libc: &Option<Libc>) -> Result<regex::Regex> {
    let libc = match libc {
        None => "",
        Some(Libc::Glibc) => "-glibc",
        Some(Libc::Musl) => "-musl",
    };
    match (std::env::consts::ARCH, std::env::consts::OS) {
        ("x86", "windows") => {
            Regex::new("otp_win32_.*.exe").wrap_err("Unable to create asset regex")
        }
        ("x86_64", "windows") => {
            Regex::new("otp_win64_.*.exe").wrap_err("Unable to create asset regex")
        }
        ("x86_64", "macos") => {
            Regex::new("otp-x86_64-apple-darwin.tar.gz").wrap_err("Unable to create asset regex")
        }
        ("aarch64", "macos") => {
            Regex::new("otp-aarch64-apple-darwin.tar.gz").wrap_err("Unable to create asset regex")
        }
        ("aarch64", "linux") => Regex::new(format!("erlang-.*-arm64{libc}.tar.gz").as_str())
            .wrap_err("Unable to create asset regex"),
        ("x86_64", "linux") => Regex::new(format!("erlang-.*-x64{libc}.tar.gz").as_str())
            .wrap_err("Unable to create asset regex"),
        _ => Err(eyre!("Unknown architecture or OS for installing Erlang")),
    }
}
