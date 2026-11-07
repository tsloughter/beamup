use crate::github;
use crate::installable::{Installable, Libc};
use crate::utils;
use color_eyre::eyre::{eyre, Report, Result, WrapErr};
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use tar::Archive;
use tempdir::TempDir;
use zip;

#[cfg(windows)]
use std::process::ExitStatus;

pub fn run<T: Installable>(
    installable: &T,
    id: &String,
    release: &str,
    libc: &Option<Libc>,
    force: bool,
) -> Result<String, Report> {
    let release_dir = installable.release_dir(id)?;
    utils::check_release_dir(&release_dir, force)?;

    let github_repo = installable.binary_repo();
    let out_dir = TempDir::new(github_repo.repo.as_str())?;
    let asset_name = installable.asset_prefix(libc)?;
    let file = github::download_asset(&asset_name, out_dir.path(), &github_repo, release)?;
    debug!("file {:?} downloaded", file);
    let extract_dir = installable.extract_dir(id)?;

    install_asset(&file, &release_dir, &extract_dir, force)
}

fn install_asset(
    file: &Path,
    release_dir: &Path,
    extract_dir: &Path,
    force: bool,
) -> Result<String> {
    let open_file = File::open(file)
        .wrap_err_with(|| format!("Downloaded Github asset {:?} not found", file))?;
    utils::maybe_create_release_dir(&release_dir.to_path_buf(), force)?;

    // TODO: better ways to check the type than the extension
    let ext = file.extension().map_or("", |e| e.to_str().unwrap_or(""));
    match ext {
        "exe" => {
            exe_run(file.to_path_buf(), release_dir.to_str().unwrap().to_owned())?;
        }
        "zip" => {
            let mut archive = zip::ZipArchive::new(open_file)?;
            archive.extract(extract_dir)?;
        }
        "gz" | "tgz" => {
            let tar = GzDecoder::new(open_file);
            let mut archive = Archive::new(tar);
            archive.unpack(extract_dir)?;
        }
        _ => {
            let install_file = extract_dir.join(
                file.file_name()
                    .ok_or_else(|| eyre!("Downloaded asset {:?} has no file name", file))?,
            );
            fs::create_dir_all(extract_dir)?;
            fs::copy(file, &install_file).wrap_err_with(|| {
                format!(
                    "Failed to copy {} to {}",
                    file.display(),
                    install_file.display()
                )
            })?;
            set_permissions(&install_file)?;
        }
    }

    Ok(release_dir.to_str().unwrap().to_owned())
}

#[cfg(unix)]
fn set_permissions(to: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(to, fs::Permissions::from_mode(0o744))?;
    Ok(())
}

#[cfg(windows)]
fn set_permissions(_to: &Path) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn exe_run(_cmd: PathBuf, _release_dir: String) -> Result<(), Report> {
    Err(eyre!(
        "Attempted to execute a Windows exeutable on a non-Windows system"
    ))
}

// thanks rustup command.rs
#[cfg(windows)]
fn exe_run(file: PathBuf, release_dir: String) -> Result<ExitStatus, Report> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    use windows_sys::Win32::Foundation::{BOOL, FALSE, TRUE};
    use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;

    unsafe extern "system" fn ctrlc_handler(_: u32) -> BOOL {
        // Do nothing. Let the child process handle it.
        TRUE
    }
    unsafe {
        if SetConsoleCtrlHandler(Some(ctrlc_handler), TRUE) == FALSE {
            return Err(eyre!("Unable to set console handler",));
        }
    }

    let mut binding = Command::new(file);
    let cmd = binding.raw_arg(&format!("/S /D={release_dir:}"));
    debug!("Command being run: {cmd:?}");

    Ok(cmd.status()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{components::Kind, languages::Language};
    use flate2::{write::GzEncoder, Compression};
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    const CONTENTS: &[u8] = b"#!/bin/sh\nexit 0\n";

    fn zip_asset(dir: &Path, name: &str) -> Result<PathBuf> {
        let file = dir.join("release.zip");
        let mut archive = zip::ZipWriter::new(File::create(&file)?);
        archive.start_file(name, SimpleFileOptions::default().unix_permissions(0o755))?;
        archive.write_all(CONTENTS)?;
        archive.finish()?;
        Ok(file)
    }

    fn tar_asset(dir: &Path, name: &str) -> Result<PathBuf> {
        let file = dir.join("release.tar.gz");
        let encoder = GzEncoder::new(File::create(&file)?, Compression::default());
        let mut archive = tar::Builder::new(encoder);
        let mut header = tar::Header::new_gnu();
        header.set_size(CONTENTS.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        archive.append_data(&mut header, name, CONTENTS)?;
        archive.into_inner()?.finish()?;
        Ok(file)
    }

    // Keep the real installable's extraction layout, but install only in the temp directory.
    fn install_fixture(
        installable: &impl Installable,
        file: &Path,
        root: &Path,
        force: bool,
    ) -> Result<String> {
        install_fixture_with_id(installable, file, root, &"custom-id".to_owned(), force)
    }

    fn install_fixture_with_id(
        installable: &impl Installable,
        file: &Path,
        root: &Path,
        id: &String,
        force: bool,
    ) -> Result<String> {
        let release_dir = installable.release_dir(id)?;
        let extract_dir = installable.extract_dir(id)?;
        let local_release_dir = root.join(release_dir.file_name().unwrap());
        let local_extract_dir = local_release_dir.join(extract_dir.strip_prefix(release_dir)?);
        install_asset(file, &local_release_dir, &local_extract_dir, force)
    }

    fn assert_executable(file: &Path) -> Result<()> {
        assert_eq!(fs::read(file)?, CONTENTS);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_ne!(fs::metadata(file)?.permissions().mode() & 0o100, 0);
        }
        Ok(())
    }

    #[test]
    fn component_zip_extracts_into_bin() -> Result<()> {
        let dir = TempDir::new("beamup-test")?;
        let file = zip_asset(dir.path(), "elp")?;

        let installed = install_fixture(&Kind::Elp, &file, dir.path(), false)?;

        assert_eq!(Path::new(&installed), dir.path().join("custom-id"));
        assert_executable(&Path::new(&installed).join("bin/elp"))?;
        assert!(!Path::new(&installed).join("elp").exists());
        Ok(())
    }

    #[test]
    fn component_tar_extracts_into_bin() -> Result<()> {
        for (kind, name) in [(Kind::Elp, "elp"), (Kind::Rebar3, "rebar3")] {
            let dir = TempDir::new("beamup-test")?;
            let file = tar_asset(dir.path(), name)?;

            let installed = install_fixture(&kind, &file, dir.path(), false)?;

            assert_executable(&Path::new(&installed).join("bin").join(name))?;
        }
        Ok(())
    }

    #[test]
    fn standalone_component_is_copied_and_made_executable() -> Result<()> {
        let dir = TempDir::new("beamup-test")?;
        let file = dir.path().join("rebar3");
        fs::write(&file, CONTENTS)?;

        let installed = install_fixture(&Kind::Rebar3, &file, dir.path(), false)?;

        assert_executable(&Path::new(&installed).join("bin/rebar3"))?;
        Ok(())
    }

    #[test]
    fn same_component_release_can_be_installed_under_multiple_ids() -> Result<()> {
        for kind in [Kind::Elp, Kind::Rebar3] {
            let dir = TempDir::new("beamup-test")?;
            let name = kind.to_string();
            let file = match kind {
                Kind::Elp => zip_asset(dir.path(), &name)?,
                Kind::Rebar3 => {
                    let file = dir.path().join(&name);
                    fs::write(&file, CONTENTS)?;
                    file
                }
            };
            let release = "same-release".to_owned();
            let mut config = crate::config::ComponentConfig::default();

            for id in ["first", "second"] {
                let id = id.to_owned();
                let installed = install_fixture_with_id(&kind, &file, dir.path(), &id, false)?;
                assert_eq!(Path::new(&installed), dir.path().join(&id));
                assert_executable(&Path::new(&installed).join("bin").join(&name))?;
                config = crate::config::update_component_config(
                    &kind, &id, &release, installed, config,
                )?;
            }

            let config = toml::Value::try_from(config)?;
            for id in ["first", "second"] {
                assert_eq!(
                    config["installs"][id]["release"].as_str(),
                    Some(release.as_str())
                );
                assert_eq!(
                    config["installs"][id]["dir"].as_str(),
                    dir.path().join(id).to_str()
                );
            }
            assert!(!dir.path().join(&release).exists());

            // Forcing one ID must not remove or modify the other installation.
            let second_marker = dir.path().join("second/keep");
            fs::write(&second_marker, b"keep")?;
            install_fixture_with_id(&kind, &file, dir.path(), &"first".to_owned(), true)?;
            assert_eq!(fs::read(second_marker)?, b"keep");
            assert_executable(&dir.path().join("second/bin").join(&name))?;
        }
        Ok(())
    }

    #[test]
    fn language_archives_keep_their_layouts() -> Result<()> {
        for (language, name) in [
            (Language::Elixir, "bin/elixir"),
            (Language::Erlang, "bin/erl"),
            (Language::Gleam, "gleam"),
        ] {
            let dir = TempDir::new("beamup-test")?;
            let file = match language {
                Language::Elixir => zip_asset(dir.path(), name)?,
                _ => tar_asset(dir.path(), name)?,
            };

            let installed = install_fixture(&language, &file, dir.path(), false)?;

            let executable = Path::new(name).file_name().unwrap();
            assert_executable(&Path::new(&installed).join("bin").join(executable))?;
        }
        Ok(())
    }

    #[test]
    fn existing_install_requires_force() -> Result<()> {
        let dir = TempDir::new("beamup-test")?;
        let file = dir.path().join("rebar3");
        fs::write(&file, CONTENTS)?;
        let release_dir = dir.path().join("custom-id");
        fs::create_dir_all(&release_dir)?;
        let marker = release_dir.join("old-install");
        fs::write(&marker, b"keep")?;

        let error = install_fixture(&Kind::Rebar3, &file, dir.path(), false).unwrap_err();
        assert!(error
            .to_string()
            .contains("Install directory already exists"));
        assert_eq!(fs::read(&marker)?, b"keep");

        install_fixture(&Kind::Rebar3, &file, dir.path(), true)?;
        assert!(!marker.exists());
        assert_executable(&release_dir.join("bin/rebar3"))?;
        Ok(())
    }

    #[test]
    fn missing_asset_does_not_replace_existing_install() -> Result<()> {
        let dir = TempDir::new("beamup-test")?;
        let release_dir = dir.path().join("custom-id");
        fs::create_dir_all(&release_dir)?;
        let marker = release_dir.join("old-install");
        fs::write(&marker, b"keep")?;

        let result = install_fixture(&Kind::Rebar3, &dir.path().join("missing"), dir.path(), true);

        assert!(result.is_err());
        assert_eq!(fs::read(marker)?, b"keep");
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn windows_installer_is_rejected_on_unix() -> Result<()> {
        let dir = TempDir::new("beamup-test")?;
        let file = dir.path().join("otp.exe");
        fs::write(&file, b"Windows installer")?;

        let error = install_fixture(&Language::Erlang, &file, dir.path(), false).unwrap_err();

        assert!(error.to_string().contains("non-Windows system"));
        Ok(())
    }
}
