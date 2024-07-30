use clap::ValueEnum;

#[derive(ValueEnum, Debug, Clone)]
pub enum Language {
    Erlang,
    Gleam,
}

pub fn get_github_org_repo(language: &Language) -> (&str, &str) {
    match language {
        Language::Erlang => ("erlang", "otp"),
        Language::Gleam => ("gleam-lang", "gleam"),
    }
}
