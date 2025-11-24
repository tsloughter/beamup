use crate::config;
use crate::languages::Language;
use color_eyre::eyre::{eyre, Result, WrapErr};
use regex::Regex;

#[cfg(unix)]
pub fn bins() -> Vec<(String, Language)> {
    vec![
        ("elixir".to_string(), Language::Elixir),
        ("elixirc".to_string(), Language::Elixir),
        ("iex".to_string(), Language::Elixir),
        ("mix".to_string(), Language::Elixir),
    ]
}

#[cfg(windows)]
pub fn bins() -> Vec<(String, Language)> {
    vec![
        ("elixir.bat".to_string(), Language::Elixir),
        ("elixirc.bat".to_string(), Language::Elixir),
        ("iex.bat".to_string(), Language::Elixir),
        ("mix.bat".to_string(), Language::Elixir),
        ("mix.ps1".to_string(), Language::Elixir),
    ]
}

pub fn asset_prefix(_release: &str) -> Result<regex::Regex> {
    // find dir of active Erlang
    match config::get_otp_major_vsn() {
        Ok(otp_major_vsn) => Regex::new(format!("elixir-otp-{otp_major_vsn:}.zip").as_str())
            .wrap_err("Unable to create regex for elixir asset"),
        Err(_) => Err(eyre!("No Erlang install found.")),
    }
}
