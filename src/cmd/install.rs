use crate::github;
use crate::languages;
use crate::utils;
use color_eyre::{eyre::eyre, eyre::Report, eyre::Result, eyre::WrapErr};
use flate2::read::GzDecoder;
use std::fs::File;
use std::path::PathBuf;
use tar::Archive;
use tempdir::TempDir;
use zip;

#[cfg(windows)]
use std::process::ExitStatus;

pub fn run(
    language: &languages::LanguageStruct,
    release: &str,
    force: bool,
) -> Result<String, Report> {
    let release_dir = &language.release_dir;
    utils::check_release_dir(release_dir, force)?;
    let github_repo = &language.binary_repo;
    let out_dir = TempDir::new(github_repo.repo.as_str())?;
    let asset_name = &language.asset_prefix;
    let file = github::download_asset(asset_name, out_dir.path(), github_repo, release)?;
    debug!("file {:?} downloaded", file);
    let open_file = File::open(&file).wrap_err_with(|| {
        format!(
            "Downloaded Github asset for release {} into file {:?} not found",
            release, &file
        )
    })?;

    utils::maybe_create_release_dir(release_dir, force)?;

    let extract_dir = &language.extract_dir;

    // TODO: better ways to check the type than the extension
    let ext = file.extension().map_or("", |e| e.to_str().unwrap_or(""));
    match ext {
        "exe" => {
            let release_dir = release_dir.clone().into_os_string().into_string().unwrap();
            exe_run(file, release_dir.clone())?;
            Ok(release_dir)
        }
        "zip" => {
            let mut archive = zip::ZipArchive::new(open_file)?;
            archive.extract(extract_dir)?;
            Ok(release_dir.clone().into_os_string().into_string().unwrap())
        }
        _ => {
            let tar = GzDecoder::new(open_file);
            let mut archive = Archive::new(tar);
            archive.unpack(extract_dir)?;
            Ok(release_dir.clone().into_os_string().into_string().unwrap())
        }
    }
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
