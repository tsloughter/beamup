use crate::eyre;
use color_eyre::eyre::Result;
use std::fs;
use std::path::PathBuf;

pub fn check_release_dir(release_dir: &PathBuf, force: bool) -> Result<()> {
    match release_dir.try_exists() {
        Ok(true) =>
            match force {
                true => {
                    Ok(())
                },
                _ => Err(eyre!("Install directory already exists. Use `-f true` to delete {:?} and recreate instead of giving this error.", release_dir)),
            }
        Ok(false) => Ok(()),
        Err(e) => Err(eyre!(
            "Unable to check for existence of install directory: {e:?}"
        )),
    }
}

pub fn maybe_create_release_dir(release_dir: &PathBuf, force: bool) -> Result<()> {
    match release_dir.try_exists() {
        Ok(true) =>
            match force {
                true => {
                    info!("Force enabled. Deleting existing release directory {:?}", release_dir);
                    fs::remove_dir_all(release_dir)?
                },
                _ => return Err(eyre!("Install directory already exists. Use `-f true` to delete {:?} and recreate instead of giving this error.", release_dir)),
            }
        Ok(false) => {},
        Err(e) => return Err(eyre!(
            "Unable to check for existence of install directory: {e:?}"
        )),
    };

    let _ = std::fs::create_dir_all(release_dir);

    Ok(())
}
