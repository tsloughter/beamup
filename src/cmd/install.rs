use crate::config;
use crate::github::{download_asset, GithubRepo};
use crate::languages;
use color_eyre::{eyre::Report, eyre::Result, eyre::WrapErr};
use flate2::read::GzDecoder;
use std::fs::File;
use tar::Archive;
use tempdir::TempDir;

pub fn run(
    language: &languages::Language,
    github_repo: &GithubRepo,
    release: &str,
    id: &String,
    _repo: &Option<String>,
    _force: &Option<bool>,
) -> Result<String, Report> {
    let out_dir = TempDir::new(github_repo.repo.as_str())?;
    let file = download_asset(language, out_dir.path(), github_repo, release)?;

    let tar_gz = File::open(&file).wrap_err_with(|| {
        format!(
            "Downloaded Github asset for release {} into file {:?} not found",
            release, &file
        )
    })?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    let release_dir = config::language_release_dir(language.to_owned(), id.to_owned());
    archive.unpack(&release_dir.join("bin"))?;

    Ok(release_dir.into_os_string().into_string().unwrap())
}
