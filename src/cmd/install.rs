use crate::config;
use crate::github::{download_asset, GithubRepo};
use crate::languages;
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
    language: &languages::Language,
    github_repo: &GithubRepo,
    release: &str,
    id: &String,
    _repo: &Option<String>,
    force: &Option<bool>,
) -> Result<String, Report> {
    let release_dir = config::language_release_dir(language, id, force)?;

    let out_dir = TempDir::new(github_repo.repo.as_str())?;
    let file = download_asset(language, out_dir.path(), github_repo, release)?;
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
        "exe" => {
            let release_dir = release_dir.into_os_string().into_string().unwrap();
            exe_run(file, release_dir.clone())?;
            Ok(release_dir)
        }
        "zip" => {
            let mut archive = zip::ZipArchive::new(open_file)?;
            archive.extract(&release_dir.join("bin"))?;
            Ok(release_dir.into_os_string().into_string().unwrap())
        }
        _ => {
            let tar = GzDecoder::new(open_file);
            let mut archive = Archive::new(tar);
            archive.unpack(&release_dir.join("bin"))?;
            Ok(release_dir.into_os_string().into_string().unwrap())
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
