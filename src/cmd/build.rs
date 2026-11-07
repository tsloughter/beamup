use crate::git::GitRef;
use crate::github::download_release_tarball;
use crate::installable::Buildable;
use crate::{
    config,
    installs::{Metadata, PendingInstall},
};
use color_eyre::eyre::{eyre, Result, WrapErr};
use console::Emoji;
use flate2::read::GzDecoder;
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use std::time::Instant;
use tar::Archive;
use tempdir::TempDir;

// http://unicode.org/emoji/charts/full-emoji-list.html
static CHECKMARK: Emoji = Emoji("✅", "✅ ");
static FAIL: Emoji = Emoji("❌", "❌ ");
static WARNING: Emoji = Emoji("🚫", "🚫");

#[derive(Copy, Clone)]
enum BuildResult {
    Success,
    Fail,
}

pub fn run<T: Buildable>(
    installable: &T,
    git_ref: &GitRef,
    id: &str,
    _repo: &Option<String>,
    force: bool,
    config: &config::Config,
) -> Result<()> {
    debug!("Building from source from git ref={git_ref} with id={id}");

    let release_dir = installable.release_dir(id, config)?;
    let pending = PendingInstall::new(&release_dir, force)?;

    //maybe grab configure options from environment
    let key = "BEAMUP_BUILD_OPTIONS";
    let user_build_options = match env::var(key) {
        Ok(options) => options,
        _ => installable.default_build_options(config),
    };

    let github_repo = installable.source_repo();
    let release = git_ref.to_string();

    let out_dir = TempDir::new(github_repo.repo.as_str())?;
    let file = download_release_tarball(out_dir.path(), &github_repo, &release)?;

    let tar_gz = File::open(&file).wrap_err_with(|| {
        format!(
            "Downloaded Github release tarball {} into file {:?} not found",
            git_ref, file
        )
    })?;

    debug!("unpacking source tarball {tar_gz:?} to {out_dir:?}");
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    let unpack_dir = out_dir.path().join("unpack");
    std::fs::create_dir_all(&unpack_dir)?;
    archive.unpack(&unpack_dir)?;

    let mut paths = std::fs::read_dir(&unpack_dir)?;
    let binding = paths
        .next()
        .ok_or_else(|| eyre!("Source archive is empty"))??
        .path();
    let unpacked_dir: &Path = binding.as_path();
    let payload = build(
        &release_dir,
        pending.staging_dir(),
        unpacked_dir,
        &user_build_options,
    )?;
    pending.publish(
        &payload,
        &Metadata {
            release,
            build_options: Some(user_build_options),
            ..Metadata::default()
        },
    )
}

fn build(
    install_dir: &Path,
    staging_dir: &Path,
    dir: &Path,
    user_build_options: &str,
) -> Result<PathBuf> {
    let mut build_options = shell_words::split(user_build_options)?;
    // The configured prefix must be the final path. DESTDIR stages the files
    // without embedding a temporary prefix in Erlang's scripts and symlinks.
    if build_options.iter().any(|option| {
        ["--prefix", "--exec-prefix", "--bindir", "--libdir"]
            .iter()
            .any(|name| {
                option == name
                    || option
                        .strip_prefix(name)
                        .is_some_and(|rest| rest.starts_with('='))
            })
    }) {
        return Err(eyre!("Install path configure options are managed by beamup; set install_dir in the config instead"));
    }
    build_options.push(format!("--prefix={}", install_dir.display()));
    let num_cpus = num_cpus::get().to_string();
    let destdir = staging_dir.join("destdir");
    let payload = destdir.join(install_dir.strip_prefix("/")?);

    // Start the spinner only once every early-return check has passed.
    let spinner_style = ProgressStyle::default_spinner()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
        .template("{prefix:.bold.dim} {spinner} {wide_msg}")?;
    let pb = ProgressBar::new_spinner();
    pb.set_style(spinner_style);
    pb.enable_steady_tick(Duration::from_millis(100));
    let steps = [
        ("./configure", build_options),
        ("make", vec!["-j".to_owned(), num_cpus.clone()]),
        (
            "make",
            vec![
                "-j".to_owned(),
                num_cpus.clone(),
                "docs".to_owned(),
                "DOC_TARGETS=chunks".to_owned(),
            ],
        ),
        (
            "make",
            vec![
                "-j".to_owned(),
                num_cpus,
                "install".to_owned(),
                format!("DESTDIR={}", destdir.display()),
            ],
        ),
    ];
    for (command, args) in steps {
        if let BuildResult::Fail = exec(command, &args, dir, Instant::now(), &pb) {
            pb.finish_and_clear();
            return Err(eyre!("Build step failed: {command} {}", args.join(" ")));
        }
        if command == "./configure" && !has_openssl(dir) {
            pb.println(format!(
                " {} No usable OpenSSL found; specify --with-ssl to enable crypto",
                WARNING
            ));
        }
    }
    pb.finish_and_clear();
    Ok(payload)
}

fn has_openssl(src_dir: &Path) -> bool {
    // check that lib/crypto/SKIP doesn't exist,
    // if it does it means something went wrong with OpenSSL
    !src_dir.join("./lib/crypto/SKIP").exists()
}

fn exec(
    command: &str,
    args: &Vec<String>,
    dir: &Path,
    started_ts: Instant,
    pb: &ProgressBar,
) -> BuildResult {
    debug!("Running {} {:?}", command, args);
    pb.set_message(format!("{} {}", command, args.join(" ")));
    match Command::new(command).args(args).current_dir(dir).output() {
        Err(e) => {
            pb.println(format!(" {} {} {}", FAIL, command, args.join(" ")));
            error!("build failed: {}", e);
            BuildResult::Fail
        }
        Ok(output) => {
            debug!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            debug!("stderr: {}", String::from_utf8_lossy(&output.stderr));

            match output.status.success() {
                true => {
                    pb.println(format!(
                        " {} {} {} (done in {})",
                        CHECKMARK,
                        command,
                        args.join(" "),
                        HumanDuration(started_ts.elapsed())
                    ));
                    BuildResult::Success
                }
                false => {
                    error!("stdout: {}", String::from_utf8_lossy(&output.stdout));
                    pb.println(format!(" {} {} {}", FAIL, command, args.join(" ")));
                    BuildResult::Fail
                }
            }
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::{fs, os::unix::fs::PermissionsExt};

    fn source_fixture(dir: &Path, fail_install: bool) -> Result<()> {
        fs::create_dir_all(dir)?;
        let configure = dir.join("configure");
        fs::write(&configure, "#!/bin/sh\nfor arg do\n case \"$arg\" in --prefix=*) printf '%s\\n' \"${arg#--prefix=}\" > prefix ;; esac\ndone\n")?;
        fs::set_permissions(configure, fs::Permissions::from_mode(0o755))?;
        let ending = if fail_install { "\tfalse\n" } else { "" };
        fs::write(dir.join("Makefile"), format!(
            ".PHONY: all docs install\nall docs:\n\t@true\ninstall:\n\tmkdir -p \"$(DESTDIR)$$(cat prefix)/bin\"\n\tcp prefix \"$(DESTDIR)$$(cat prefix)/bin/erl\"\n{ending}"
        ))?;
        Ok(())
    }

    #[test]
    fn staged_build_uses_final_prefix_and_publishes_metadata() -> Result<()> {
        let root = TempDir::new("beamup-build")?;
        let source = root.path().join("source");
        source_fixture(&source, false)?;
        let destination = root.path().join("custom root/erlang/debug");
        let pending = PendingInstall::new(&destination, false)?;
        let payload = build(
            &destination,
            pending.staging_dir(),
            &source,
            "--enable-lock-counter",
        )?;
        assert!(!destination.exists());
        assert_eq!(
            fs::read_to_string(payload.join("bin/erl"))?.trim(),
            destination.to_str().unwrap()
        );
        pending.publish(
            &payload,
            &Metadata {
                release: "OTP-28.1".to_owned(),
                build_options: Some("--enable-lock-counter".to_owned()),
                ..Metadata::default()
            },
        )?;
        assert!(destination.join("bin/erl").is_file());
        let metadata: Metadata = toml::from_str(&fs::read_to_string(
            destination.join(crate::installs::METADATA_FILE),
        )?)?;
        assert_eq!(
            metadata.build_options.as_deref(),
            Some("--enable-lock-counter")
        );
        Ok(())
    }

    #[test]
    fn failed_make_install_preserves_previous_install_and_cleans_staging() -> Result<()> {
        let root = TempDir::new("beamup-build-failure")?;
        let source = root.path().join("source");
        source_fixture(&source, true)?;
        let destination = root.path().join("erlang/debug");
        fs::create_dir_all(destination.join("bin"))?;
        fs::write(destination.join("bin/erl"), b"previous install")?;
        {
            let pending = PendingInstall::new(&destination, true)?;
            let error = build(&destination, pending.staging_dir(), &source, "").unwrap_err();
            assert!(error.to_string().contains("Build step failed: make"));
            assert_eq!(fs::read(destination.join("bin/erl"))?, b"previous install");
        }
        assert_eq!(fs::read_dir(destination.parent().unwrap())?.count(), 1);
        Ok(())
    }
}
