use crate::github::GithubRepo;
use crate::languages;
//use crate::languages::gleam;
use clap::ValueEnum;
use std::process;

pub mod erlang;
pub mod gleam;

pub const BIN_MAP: &[(&str, languages::Language)] = &[
    ("gleam", languages::Language::Gleam),
    ("ct_run", languages::Language::Erlang),
    ("dialyzer", languages::Language::Erlang),
    ("epmd", languages::Language::Erlang),
    ("erl", languages::Language::Erlang),
    ("erlc", languages::Language::Erlang),
    ("erl_call", languages::Language::Erlang),
    ("escript", languages::Language::Erlang),
    ("run_erl", languages::Language::Erlang),
    ("run_test", languages::Language::Erlang),
    ("to_erl", languages::Language::Erlang),
    ("typer", languages::Language::Erlang),
];

#[derive(ValueEnum, Debug, Clone, PartialEq)]
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

pub fn get_github_repo(language: &Language) -> GithubRepo {
    match language {
        Language::Gleam => gleam::get_github_repo(),
        Language::Erlang => erlang::get_github_repo(),
    }
}

pub fn bin_to_language(bin: &str) -> &languages::Language {
    match languages::BIN_MAP.iter().find(|&(k, _)| *k == bin) {
        Some((_, language)) => language,
        _ => {
            error!("No language to run command {bin} for found");
            process::exit(1)
        }
    }
}
