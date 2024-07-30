use crate::github;
use crate::languages;

pub fn install(
    language: &languages::Language,
    release: &String,
    _id: &Option<String>,
    _repo: &Option<String>,
    _force: &Option<bool>,
) {
    github::download_asset(language, release);
}
