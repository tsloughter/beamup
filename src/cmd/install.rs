use crate::config;
use crate::github::{download_asset, GithubRepo};
use crate::languages;
use color_eyre::{eyre::Report, eyre::Result, eyre::WrapErr};
use flate2::read::GzDecoder;
use std::fs::File;
use tar::Archive;
use tempdir::TempDir;
use zip;

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
    let release_dir = config::language_release_dir(language.to_owned(), id.to_owned());
    debug!("file {:?} downloaded", file);
    let open_file = File::open(&file).wrap_err_with(|| {
        format!(
            "Downloaded Github asset for release {} into file {:?} not found",
            release, &file
        )
    })?;

    // TODO: better ways to check the type than the extension
    let ext = file.extension().map_or("", |e| e.to_str().unwrap_or(""));
    match ext {
        "zip" => {
            let mut archive = zip::ZipArchive::new(open_file)?; 
            archive.extract(&release_dir.join("bin"))?;
        },
        _ => {
            let tar = GzDecoder::new(open_file);
            let mut archive = Archive::new(tar);
            archive.unpack(&release_dir.join("bin"))?;
    }
};

    Ok(release_dir.into_os_string().into_string().unwrap())
}
