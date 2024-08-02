use clap::ValueEnum;

#[derive(ValueEnum, Debug, Clone)]
pub enum Language {
    Erlang,
    Gleam,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Language::Erlang => write!(f, "erlang"),
            Language::Gleam => write!(f, "gleam"),
        }
    }
}

pub fn get_github_org_repo(language: &Language) -> (&str, &str) {
    match language {
        Language::Erlang => ("erlang", "otp"),
        Language::Gleam => ("gleam-lang", "gleam"),
    }
}
