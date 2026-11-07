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

pub fn otp_major_vsn(config: &config::Config) -> Result<String> {
    match std::env::var("BEAMUP_OTP_MAJOR_VSN") {
        Ok(value) => {
            let value = value.trim();
            if value.is_empty() {
                return Err(eyre!(
                    "Environment variable BEAMUP_OTP_MAJOR_VSN is set but empty"
                ));
            }
            Ok(value.to_string())
        }
        Err(std::env::VarError::NotPresent) => config::get_otp_major_vsn(config),
        Err(err) => Err(eyre!(err).wrap_err("Unable to read BEAMUP_OTP_MAJOR_VSN")),
    }
}

pub fn asset_prefix(otp_major_vsn: &str) -> Result<regex::Regex> {
    Regex::new(format!("elixir-otp-{otp_major_vsn}.zip").as_str())
        .wrap_err("Unable to create regex for elixir asset")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn otp_major_vsn_uses_env_override() {
        std::env::set_var("BEAMUP_OTP_MAJOR_VSN", "999");
        let config = toml::from_str("install_dir = '/unused'").unwrap();
        let otp_major_vsn = otp_major_vsn(&config).expect("expected version");
        std::env::remove_var("BEAMUP_OTP_MAJOR_VSN");
        assert_eq!(otp_major_vsn, "999");
        let asset_prefix = asset_prefix(&otp_major_vsn).expect("expected regex");
        assert!(asset_prefix.is_match("elixir-otp-999.zip"));
    }
}
