extern crate clap;

#[macro_use]
extern crate log;
#[macro_use]
extern crate strum;

use clap::{Args, Command, CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Generator, Shell};
use console::style;
use log::{Level, LevelFilter, Record};
use std::env;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process;

mod config;

use color_eyre::{config::HookBuilder, eyre::eyre, eyre::Report, eyre::Result};

mod cmd;
mod components;
mod git;
mod github;
mod languages;
mod links;
mod run;
mod utils;

#[derive(Parser, Debug)]
#[command(version, about = "Manage BEAM language installs.", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(short, long)]
    config: Option<String>,

    #[command(subcommand)]
    subcommand: SubCommands,
}

#[derive(Subcommand, Debug)]
enum SubCommands {
    Generate(GenerateArgs),

    /// List supported languages
    Languages,

    /// List supported components
    Components,

    /// Update binary symlinks to erlup executable
    UpdateLinks,

    /// List installed languages
    List,

    /// Fetch available releases for language
    Releases(ReleasesArgs),

    /// Fetch latest tags for repo
    Fetch(RepoArgs),

    /// List available tags to build for a repo
    Tags(RepoArgs),

    /// List available branches to build for a repo
    Branches(RepoArgs),

    /// Switch install to use by id
    Switch(IdArgs),

    /// Set default install to use by id
    Default(IdArgs),

    /// Deletes an install by id
    Delete(IdArgs),

    /// Build and install by branch of tag name
    Build(BuildArgs),

    /// Install binary release of language
    Install(InstallArgs),

    /// Manage components
    Component(ComponentSubCommands),

    /// Update repos to the config
    Repo(RepoSubCommands),

    /// Add or remove a link to an existing install
    Link(LinkSubCommands),
}

#[derive(Args, Debug)]
struct GenerateArgs {
    // Shell to generate completions for
    shell: Shell,
}

#[derive(Args, Debug)]
struct ReleasesArgs {
    /// Language to list releases for
    language: languages::Language,

    /// Which repo to use for command
    #[arg(short, long)]
    repo: Option<String>,
}

#[derive(Args, Debug)]
struct RepoArgs {
    /// Which repo to use for command
    #[arg(short, long)]
    repo: Option<String>,
}

#[derive(Args, Debug)]
struct IdArgs {
    /// Language to use
    language: languages::Language,

    /// Id of the install
    id: String,
}

#[derive(Args, Debug)]
struct BuildArgs {
    /// Language to build a release or branch of
    language: languages::Language,

    /// Release to build
    release: Option<String>,

    /// Branch or tag of the repo
    #[arg(short, long)]
    branch: Option<String>,

    /// Id to give the build
    #[arg(short, long)]
    id: Option<String>,

    /// Which repo to use for command
    #[arg(short, long)]
    repo: Option<String>,

    /// Forces a build disregarding any previously existing ones
    #[arg(short, long)]
    force: bool,
}

#[derive(Args, Debug)]
struct InstallArgs {
    /// Language to install release of
    language: languages::Language,

    /// Release version to install
    release: String,

    /// Id to give the install
    #[arg(short, long)]
    id: Option<String>,

    /// Where to grab the releases
    #[arg(short, long)]
    repo: Option<String>,

    /// Forces an install disregarding any previously existing ones
    #[arg(short, long)]
    force: bool,

    /// For Erlang only. Select the libc the install wil be built to dynamically link against.
    #[arg(short, long)]
    libc: Option<languages::Libc>,
}

#[derive(Args, Debug)]
struct ComponentSubCommands {
    #[command(subcommand)]
    cmd: ComponentCmds,
}

#[derive(Subcommand, Debug)]
enum ComponentCmds {
    /// Install a component
    Install(ComponentInstallArgs),
}

#[derive(Args, Debug)]
struct ComponentInstallArgs {
    /// Component to install
    component: components::Kind,

    /// Release version to install
    release: String,

    /// Id to give the install
    #[arg(short, long)]
    id: Option<String>,

    /// Where to grab the releases
    #[arg(short, long)]
    repo: Option<String>,

    /// Forces an install disregarding any previously existing ones
    #[arg(short, long)]
    force: bool,
}

#[derive(Args, Debug)]
struct RepoSubCommands {
    #[command(subcommand)]
    cmd: RepoCmds,
}

#[derive(Subcommand, Debug)]
enum RepoCmds {
    /// Add repo to the configuration
    Add(RepoAddArgs),

    /// Remove repo from the configuration
    Rm(RepoRmArgs),

    /// List available repos
    Ls,
}

#[derive(Args, Debug)]
struct RepoAddArgs {
    /// Language to add repo for
    language: languages::Language,

    /// id of the repo to add
    id: String,

    /// Url of the git repo for the repo
    repo: String,
}

#[derive(Args, Debug)]
struct RepoRmArgs {
    /// Language to remove repo for
    language: languages::Language,

    /// id of the repo to remove
    id: String,
}

#[derive(Args, Debug)]
struct LinkSubCommands {
    #[command(subcommand)]
    cmd: LinkCmds,
}

#[derive(Subcommand, Debug)]
enum LinkCmds {
    /// Add link to existing installation of language
    Add(LinkAddArgs),

    /// Remove link to installation language
    Rm(LinkRmArgs),
}

#[derive(Args, Debug)]
struct LinkAddArgs {
    /// Language to add existing installation for
    language: languages::Language,

    /// id of the installation to link to
    id: String,

    /// Path of the existing installation
    path: String,
}

#[derive(Args, Debug)]
struct LinkRmArgs {
    /// Language to remove existing installation for
    language: languages::Language,

    /// id of the installation to remove
    id: String,
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}

fn handle_command(_bin_path: PathBuf) -> Result<(), Report> {
    let cli = Cli::parse();

    let (config_file, config) = match &cli.config {
        Some(file) => (file.to_owned(), config::read_config(file.to_owned())),
        None => config::home_config()?,
    };

    match &cli.subcommand {
        SubCommands::Generate(GenerateArgs { shell }) => {
            debug!("Generating completion file for {shell:?}...");
            print_completions(*shell, &mut Cli::command());
            Ok(())
        }
        SubCommands::Languages => {
            debug!("running list");
            println!("Languages:\n");
            languages::print();
            Ok(())
        }
        SubCommands::Components => {
            debug!("running list");
            println!("Components:\n");
            components::print();
            Ok(())
        }
        SubCommands::List => {
            debug!("running list");
            cmd::list::run(&config);
            Ok(())
        }
        SubCommands::Releases(ReleasesArgs { language, .. }) => {
            debug!("running releases: language={:?}", language);

            // TODO: should return Result type
            cmd::releases::run(language);
            Ok(())
        }
        SubCommands::Install(InstallArgs {
            language,
            release,
            id,
            repo,
            force,
            libc,
        }) => {
            debug!(
                "running install: {:?} {} {:?} {:?} {:?} {:?}",
                language, release, id, repo, force, libc
            );

            check_if_install_supported(language)?;

            // if no user supplied id then use the name of
            // the release to install
            let id = id.as_ref().unwrap_or(release);

            info!(
                "Downloading and installing {:?} for release={} id={}",
                language, release, id
            );

            let dir = cmd::install::run(language, id, release, libc, *force)?;
            cmd::update_links::run(Some(language), &config)?;

            config::add_install(language, id, release, dir, config_file, config)?;

            info!(
                "Completed install of {:?} for release={} id={}",
                language, release, id
            );

            Ok(())
        }
        SubCommands::UpdateLinks => {
            debug!("running update-links");

            cmd::update_links::run(None, &config)?;

            info!("Updated links of language binaries to current beamup install");

            Ok(())
        }
        SubCommands::Default(IdArgs { language, id }) => {
            debug!("running default: {:?} {:?}", language, id);

            info!(
                "Setting default {:?} to use to install of id{}",
                language, id
            );

            cmd::default::run(language, id, config_file, config)
        }
        SubCommands::Switch(IdArgs { language, id }) => {
            debug!("running switch: {:?} {:?}", language, id);

            info!("Switching local {:?} to use install of id={}", language, id);

            cmd::switch::run(language, id, config)
        }
        SubCommands::Build(BuildArgs {
            language,
            release,
            branch,
            id,
            repo,
            force,
        }) => {
            debug!(
                "running build: {:?} {:?} {:?} {:?} {:?} {:?}",
                language, release, branch, id, repo, force
            );

            if *language != languages::Language::Erlang {
                return Err(eyre!(
                    "build command not supported yet for language {language:?}"
                ));
            }

            if std::env::consts::OS == "windows" {
                return Err(eyre!("build command not supported yet for Windows"));
            }

            let git_ref = match release {
                None => match branch {
                    None => {
                        return Err(eyre!(
                            "build command needs a release argument or the -b <branch> option"
                        ))
                    }
                    Some(branch) => git::GitRef::Branch(branch.to_owned()),
                },
                Some(release) => git::GitRef::Release(release.to_owned()),
            };
            let id = id.clone().unwrap_or(git_ref.to_string());

            info!("Building {:?} for ref={} id={}", language, git_ref, id);
            let dir = cmd::build::run(language, &git_ref, &id, repo, *force, &config)?;

            cmd::update_links::run(Some(language), &config)?;

            config::add_install(
                language,
                &id,
                &git_ref.to_string(),
                dir,
                config_file,
                config,
            )?;

            info!(
                "Completed build and install of {:?} for ref={} id={}",
                language, git_ref, id
            );

            Ok(())
        }
        SubCommands::Component(ComponentSubCommands {
            cmd:
                ComponentCmds::Install(ComponentInstallArgs {
                    component,
                    release,
                    id,
                    repo: _repo,
                    force,
                }),
        }) => {
            debug!("running component install {component:?}");

            check_if_component_install_supported()?;

            // if no user supplied id then use the name of
            // the release to install
            let id = id.as_ref().unwrap_or(release);

            let c = components::Component::new(component.clone(), release)?;

            let release_dir = cmd::component_install::run(&c, release, *force)?;

            let bin_dir = config::bin_dir();
            let _ = std::fs::create_dir_all(&bin_dir);

            let (bins, _): (Vec<String>, Vec<components::Kind>) = c.bins.into_iter().unzip();

            links::update(bins.into_iter(), &bin_dir)?;

            config::add_component_install(
                component,
                id,
                &release.to_string(),
                release_dir.to_string(),
                config_file,
                config,
            )?;

            info!("Completed install of component {component:?} with id={id}");

            Ok(())
        }

        _ => Err(eyre!("subcommand not implemented yet")),
    }
}

// only Elixir and Gleam support install on any platform
// Erlang only on Windows
fn check_if_install_supported(language: &languages::Language) -> Result<()> {
    if *language == languages::Language::Erlang {
        // install command for Erlang only supports Windows on x86 or x86_64 at this time
        match (std::env::consts::ARCH, std::env::consts::OS) {
            ("x86", "windows") => return Ok(()),
            ("x86_64", "windows") => return Ok(()),
            ("x86_64", "macos") => return Ok(()),
            ("aarch64", "macos") => return Ok(()),
            ("x86_64", "linux") => return Ok(()),
            ("aarch64", "linux") => return Ok(()),
            (os, arch) => {
                return Err(eyre!(
                    "install command not supported yet for language {language:?} on {os:?} {arch:?}"
                ))
            }
        }
    }

    if *language == languages::Language::Elixir {
        // catch when no Erlang is installed and made the default
        match config::get_otp_major_vsn() {
            Err(_) => return Err(eyre!("No default Erlang installation found. Install an Erlang version, like `beamup install erlang latest` or set a default with `beamup default erlang <ID>` first.")),
            _ => ()
        }
    }

    Ok(())
}

// ELP and rebar3 don't (yet) provide Windows binaries
fn check_if_component_install_supported() -> Result<()> {
    match (std::env::consts::ARCH, std::env::consts::OS) {
        (_, "windows") => Err(eyre!(
            "component install command not supported yet on Windows"
        )),
        _ => Ok(()),
    }
}

fn setup_logging() {
    let format = |buf: &mut env_logger::fmt::Formatter, record: &Record| {
        if record.level() == Level::Error {
            writeln!(buf, "{}", style(format!("{}", record.args())).red())
        } else if record.level() == Level::Info {
            writeln!(buf, "{}", record.args())
        } else {
            writeln!(buf, "{}", style(format!("{}", record.args())).blue())
        }
    };

    let key = "DEBUG";
    let level = match env::var(key) {
        Ok(_) => LevelFilter::Debug,
        _ => LevelFilter::Info,
    };

    env_logger::builder()
        .format(format)
        .filter(None, level)
        .init();
}

fn main() -> Result<(), Report> {
    // color_eyre::install()?;
    setup_logging();

    let mut args = env::args();
    let binname = args.next().unwrap();
    let f = Path::new(&binname).file_name().unwrap();

    HookBuilder::default()
        .display_location_section(false)
        .install()?;

    if f.eq("beamup") || f.eq("beamup.exe") {
        match env::current_exe() {
            Ok(bin_path) => {
                debug!("current bin path: {}", bin_path.display());
                handle_command(bin_path)
            }
            Err(e) => {
                println!("failed to get current bin path: {}", e);
                process::exit(1)
            }
        }
    } else {
        let (_, config) = config::home_config()?;
        match languages::bins(&config)
            .iter()
            .find(|&(k, _)| *k == f.to_str().unwrap())
        {
            Some((c, _)) => {
                let bin = Path::new(c).file_name().unwrap();
                run::run(bin.to_str().unwrap(), args)
            }
            None => match components::bins().iter().find(|(e, _)| e.as_str() == f) {
                Some((e, k)) => run::run_component(e, k, args),
                None => Err(eyre!("beamup found no such command: {f:?}")),
            },
        }
    }
}
