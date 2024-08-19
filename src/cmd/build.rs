use crate::config;
use crate::git::GitRef;
use crate::github::download_release_tarball;
use crate::languages::{get_github_repo, Language};
use color_eyre::{eyre::Result, eyre::WrapErr};
use console::Emoji;
use flate2::read::GzDecoder;
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use std::env;
use std::fs::File;
use std::path::Path;
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

struct CheckContext<'a> {
    src_dir: &'a Path,
    install_dir: &'a Path,
    build_status: BuildResult,
}

enum CheckResult<'a> {
    Success,
    Warning(&'a str),
    Fail,
}

enum BuildStep<'a> {
    Exec(&'a str, Vec<String>),
    Check(Box<dyn Fn(&CheckContext) -> CheckResult<'a>>),
}

pub fn run(
    language: &Language,
    git_ref: &GitRef,
    id: &String,
    _repo: &Option<String>,
    _force: &Option<bool>,
    config: &config::Config,
) -> Result<String> {
    debug!("Building {language} from source from git ref={git_ref} with id={id}");

    //maybe grab configure options from environment
    let key = "BEAMUP_BUILD_OPTIONS";
    let user_build_options = match env::var(key) {
        Ok(options) => options,
        _ => config::lookup_default_build_options(language, config),
    };

    let github_repo = get_github_repo(language);
    let release = git_ref.to_string();

    let out_dir = TempDir::new(github_repo.repo.as_str())?;
    let file = download_release_tarball(language, out_dir.path(), &github_repo, &release)?;

    let tar_gz = File::open(&file).wrap_err_with(|| {
        format!(
            "Downloaded Github release tarball {} into file {:?} not found",
            git_ref, &file
        )
    })?;

    let release_dir = config::language_release_dir(language.to_owned(), id.to_owned());
    debug!("unpacking source tarball {tar_gz:?} to {out_dir:?}");
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    let unpack_dir = out_dir.path().join("unpack");
    std::fs::create_dir_all(&unpack_dir)?;
    archive.unpack(&unpack_dir)?;

    let mut paths = std::fs::read_dir(&unpack_dir)?;
    let binding = paths.next().unwrap()?.path();
    let unpacked_dir: &Path = binding.as_path();
    std::fs::create_dir_all(&release_dir)?;
    build(&release_dir, unpacked_dir, user_build_options.as_str())?;

    Ok(release_dir.into_os_string().into_string().unwrap())
}

fn build(install_dir: &Path, dir: &Path, user_build_options0: &str) -> Result<()> {
    let num_cpus = num_cpus::get().to_string();
    let spinner_style = ProgressStyle::default_spinner()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
        .template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap();

    let pb = ProgressBar::new_spinner();
    pb.set_style(spinner_style);
    pb.enable_steady_tick(Duration::from_millis(100));

    // split the configure options into a vector of String in a shell sensitive way
    // eg.
    //  from:
    //      user_build_options0: --without-wx --without-observer --without-odbc --without-debugger --without-et --enable-builtin-zlib --without-javac CFLAGS="-g -O2 -march=native"
    //  to:
    //      user_build_options: ["--without-wx", "--without-observer", "--without-odbc", "--without-debugger", "--without-et", "--enable-builtin-zlib", "--without-javac", "CFLAGS=-g -O2 -march=native"]
    let mut user_build_options: Vec<String> = shell_words::split(user_build_options0)?;
    // basic configure options must always include a prefix
    let mut build_options = vec![
        "--prefix".to_string(),
        install_dir.to_str().unwrap().to_string(),
    ];
    // append the user defined options
    build_options.append(&mut user_build_options);

    // declare the build pipeline steps
    let build_steps: [BuildStep; 7] = [
        BuildStep::Exec("./configure", build_options),
        BuildStep::Check(Box::new(|context| {
            if has_openssl(context.src_dir) {
                CheckResult::Success
            } else {
                CheckResult::Warning("No usable OpenSSL found, please specify one with --with-ssl configure option, `crypto` application will not work in current build")
            }
        })),
        BuildStep::Exec("make", vec!["-j".to_string(), num_cpus.to_string()]),
        BuildStep::Exec(
            "make",
            vec![
                "-j".to_string(),
                num_cpus.to_string(),
                "docs".to_string(),
                "DOC_TARGETS=chunks".to_string(),
            ],
        ),
        // after `make` we'll already know if this build failed or not, this allows us
        // to make a better decision in wether to delete the installation dir should there
        // be one.
        BuildStep::Check(Box::new(|context| {
            match context.build_status {
                BuildResult::Fail => {
                    debug!("build has failed, aborting install to prevent overwriting a possibly working installation dir");
                    // this build has failed, we won't touch the previously existing install
                    // dir, for all we know it could hold a previously working installation
                    CheckResult::Fail
                }
                // if the build succeeded, then we check for an already existing
                // install dir, if we find one we can delete it and proceed to the
                // install phase
                BuildResult::Success => {
                    // is install dir empty? courtesy of StackOverflow
                    let is_empty = context
                        .install_dir
                        .read_dir()
                        .map(|mut i| i.next().is_none())
                        .unwrap_or(false);
                    if is_empty {
                        // it's fine, it was probably us who created the dir just a moment ago,
                        // that's why it's empty
                        CheckResult::Success
                    } else {
                        debug!("found a non empty installation dir after a successful build, removing it");
                        // dir is not empty, maybe a working installation is already there,
                        // delete the whole thing and proceed, we can go ahead with this
                        // because we know we have a working build in our hands
                        let _ = std::fs::remove_dir_all(context.install_dir);
                        CheckResult::Success
                    }
                }
            }
        })),
        BuildStep::Exec(
            "make",
            vec![
                "-j".to_string(),
                num_cpus.to_string(),
                "install".to_string(),
            ],
        ),
        BuildStep::Exec(
            "make",
            vec![
                "-j".to_string(),
                num_cpus.to_string(),
                "install-docs".to_string(),
            ],
        ),
    ];
    // execute them sequentially
    let mut build_status = BuildResult::Success;
    for step in build_steps.iter() {
        let step_started = Instant::now();

        match step {
            BuildStep::Exec(command, args) => {
                // it only takes one exec command to fail for the build status
                // to be fail as well, a subsequent check build step can optionally decide
                // to fail the pipeline
                if let BuildResult::Fail = exec(command, args, dir, step_started, &pb) {
                    build_status = BuildResult::Fail;
                }
            }
            BuildStep::Check(fun) => {
                let context = CheckContext {
                    src_dir: dir,
                    install_dir,
                    build_status,
                };
                match fun(&context) {
                    CheckResult::Success => {
                        debug!("success");
                    }
                    CheckResult::Warning(warning) => {
                        debug!("{}", warning);
                        pb.set_message(warning);
                        pb.println(format!(" {} {}", WARNING, warning));
                    }
                    CheckResult::Fail => {
                        // abort
                        pb.finish_and_clear();
                        std::process::exit(1);
                    }
                }
            }
        }
    }

    Ok(())
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
