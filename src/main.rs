extern crate clap;

#[macro_use]
extern crate log;
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
use color_eyre::{eyre::eyre, eyre::Report, eyre::Result};

mod git;
mod github;
mod languages;
mod run;

mod cmd;

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

    /// Update repos to the config
    Repo(RepoSubCommands),
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
    force: Option<bool>,
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
    force: Option<bool>,
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
    /// Name of the repo to add
    name: String,

    /// Url of the git repo for the repo
    repo: String,
}

#[derive(Args, Debug)]
struct RepoRmArgs {
    /// Name of the repo to remove
    name: String,
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
            println!("erlang");
            println!("gleam");
            Ok(())
        }
        SubCommands::List => {
            debug!("running list");
            Ok(())
        }
        SubCommands::Releases(ReleasesArgs { language, .. }) => {
            debug!("running releases: repo={:?}", language);
            cmd::releases::run(language);
            Ok(())
        }
        SubCommands::Install(InstallArgs {
            language,
            release,
            id,
            repo,
            force,
        }) => {
            debug!(
                "running install: {:?} {} {:?} {:?} {:?}",
                language, release, id, repo, force
            );
            // if no user supplied id then use the name of
            // the release to install
            let id = id.as_ref().unwrap_or(release);

            let github_repo = languages::get_github_repo(language);

            let dir = cmd::install::run(language, &github_repo, release, id, repo, force)?;
            cmd::update_links::run(Some(language));

            config::add_install(language, id, dir, config_file, config);

            Ok(())
        }
        SubCommands::UpdateLinks => {
            debug!("running update-links");

            cmd::update_links::run(None);

            Ok(())
        }
        SubCommands::Build(BuildArgs {
            language,
            release,
            branch,
            id,
            repo,
            force,
        }) => {
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
            let dir = cmd::build::run(language, &git_ref, &id, repo, force)?;

            cmd::update_links::run(Some(language));
            config::add_install(language, &id, dir, config_file, config);

            Ok(())
        }
        _ => Err(eyre!("subcommand not implemented yet")),
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

    if f.eq("beamup") {
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
        match languages::BIN_MAP.iter().find(|&(k, _)| *k == f) {
            Some((c, _)) => {
                let bin = Path::new(c).file_name().unwrap();
                run::run(bin.to_str().unwrap(), args)
            }
            None => Err(eyre!("beamup found no such command: {f:?}")),
        }
    }
}
