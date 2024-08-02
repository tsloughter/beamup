use flate2::read::GzDecoder;
use std::fs::File;
use std::path::*;
use tar::Archive;

use crate::config;
use crate::github;
use crate::languages;

pub fn install(
    language: &languages::Language,
    release: &String,
    maybe_id: &Option<String>,
    _repo: &Option<String>,
    _force: &Option<bool>,
) -> String {
    let id = match maybe_id {
        None => release,
        Some(id) => id,
    };

    let file = "gleam.tar.gz"; //github::download_asset(language, release).unwrap();

    let tar_gz = File::open(file).unwrap();
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    let release_dir = config::language_release_dir(language.to_owned(), id.to_owned()).join("bin");
    archive.unpack(&release_dir).unwrap();

    release_dir.into_os_string().into_string().unwrap()
}
