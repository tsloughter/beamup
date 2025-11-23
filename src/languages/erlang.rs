use crate::languages::{Language, Libc};
use color_eyre::eyre::Result;

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

pub fn asset_prefix(release: &str, libc: &Option<Libc>) -> Result<String> {
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
