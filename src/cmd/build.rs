use crate::config;
use crate::git::GitRef;
use crate::github::download_release_tarball;
use crate::languages::{get_github_repo, Language};
use color_eyre::{eyre::Result, eyre::WrapErr};
use flate2::read::GzDecoder;
use std::fs::File;
use tar::Archive;
use tempdir::TempDir;

pub fn run(
    language: &Language,
    git_ref: &GitRef,
    id: &String,
    _repo: &Option<String>,
    _force: &Option<bool>,
) -> Result<String> {
    debug!("Building {language} from source from git ref={git_ref} with id={id}");

    let github_repo = get_github_repo(language);
    let release = git_ref.to_string();

    let out_dir = TempDir::new(github_repo.repo.as_str())?;
    let file = download_release_tarball(language, &out_dir.path(), &github_repo, &release)?;

    let tar_gz = File::open(&file).wrap_err_with(|| {
        format!(
            "Downloaded Github release tarball {} into file {:?} not found",
            git_ref, &file
        )
    })?;

    let release_dir = config::language_release_dir(language.to_owned(), id.to_owned());
    debug!("unpacking source tarball {tar_gz:?} to {release_dir:?}");
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(&release_dir).unwrap();

    Ok(release_dir.into_os_string().into_string().unwrap())
}
