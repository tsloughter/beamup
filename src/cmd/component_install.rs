use crate::components;
use crate::github;
use crate::utils;
use color_eyre::{eyre::Report, eyre::Result, eyre::WrapErr};
use flate2::read::GzDecoder;
use std::fs;
use std::path::PathBuf;
use tar::Archive;
use tempdir::TempDir;
use zip;

pub fn run(
    c: &components::Component,
    release: &String,
    _id: &str,
    force: bool,
) -> Result<String, Report> {
    utils::check_release_dir(&c.release_dir, force)?;
    let release_dir_string = c
        .release_dir
        .clone()
        .into_os_string()
        .into_string()
        .unwrap();
    let asset_name = &c.asset_prefix;
    let github_repo = &c.repo;
    let out_dir = TempDir::new(github_repo.repo.as_str())?;
    let file = github::download_asset(asset_name, out_dir.path(), github_repo, release)?;
    debug!("file {:?} downloaded", file);
    let open_file = fs::File::open(&file).wrap_err_with(|| {
        format!(
            "Downloaded Github asset for release {} into file {:?} not found",
            release, &file
        )
    })?;

    utils::maybe_create_release_dir(&c.release_dir, force)?;

    // TODO: better ways to check the type than the extension
    let ext = file.extension().map_or("", |e| e.to_str().unwrap_or(""));
    match ext {
        "zip" => {
            let mut archive = zip::ZipArchive::new(open_file)?;
            let release_dir = match c.kind {
                components::Kind::Elp => c.release_dir.join("bin"),
                _ => c.release_dir.clone(),
            };
            archive.extract(&release_dir)?;
            Ok(release_dir_string)
        }
        "gz" => {
            let tar = GzDecoder::new(open_file);
            let mut archive = Archive::new(tar);
            archive.unpack(&c.release_dir.join("bin"))?;
            Ok(release_dir_string)
        }
        _ => {
            // no unpacking needed, just copy to bin dir and make sure its executable
            let install_file = &c.release_dir.join("bin").join(file.file_name().unwrap());
            let _ = std::fs::create_dir_all(c.release_dir.join("bin"));
            fs::copy(&file, install_file).wrap_err_with(|| {
                format!(
                    "Failed to copy {} to {}",
                    file.display(),
                    install_file.display()
                )
            })?;

            set_permissions(install_file)?;

            Ok(release_dir_string)
        }
    }
}

#[cfg(unix)]
fn set_permissions(to: &PathBuf) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let executable_permissions = PermissionsExt::from_mode(0o744);

    let to_file = fs::File::open(to)?;
    to_file.set_permissions(executable_permissions)?;

    Ok(())
}

#[cfg(windows)]
fn set_permissions(_to: &PathBuf) -> Result<()> {
    Ok(())
}
