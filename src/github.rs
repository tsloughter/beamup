use crate::languages::Language;
use color_eyre::{eyre::eyre, eyre::Report, eyre::Result, eyre::WrapErr};
use console::{style, Emoji};
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use std::io::Cursor;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

// http://unicode.org/emoji/charts/full-emoji-list.html
static CHECKMARK: Emoji = Emoji("✅", "✅ ");
// static FAIL: Emoji = Emoji("❌", "❌ ");
// static WARNING: Emoji = Emoji("🚫", "🚫");

pub struct GithubRepo {
    pub org: String,
    pub repo: String,
}

// pub trait Github {
//     fn constants_to_asset_suffix() -> Result<&'static str, &'static str>;
// }

fn asset_name(language: &Language, tag: &str) -> Result<String> {
    let suffix = match (std::env::consts::ARCH, std::env::consts::OS) {
        ("x86_64", "linux") => "x86_64-unknown-linux-musl.tar.gz",
        ("aarch", "linux") => "aarch-unknown-linux-musl.tar.gz",
        ("x86_64", "apple") => "x86_64-apple-darwin.tar.gz",
        ("aarch", "apple") => "aarch-apple-darwin.tar.gz",
        ("x86_64", "windows") => "x86_64-pc-windows-msvc.zip",
        (arch, os) => {
            let e: Report = eyre!("no {language} asset found to support arch:{arch} os:{os}");
            return Err(e);
        }
    };

    Ok(format!("{language}-{tag}-{suffix}"))
}

pub fn print_releases(GithubRepo { org, repo }: &GithubRepo) {
    let rt = setup_tokio();

    let releases = rt.block_on(async {
        let octocrab = octocrab::instance();
        octocrab.repos(org, repo).releases().list().send().await
    });

    match releases {
        Ok(octocrab::Page { items, .. }) => {
            for release in items.iter() {
                let octocrab::models::repos::Release { tag_name, .. } = release;
                println!("{tag_name}");
            }
        }
        _ => {
            error!("Failed to fetch releases for ");
        }
    }
}

pub fn download_asset(
    language: &Language,
    out_dir: &Path,
    GithubRepo { org, repo }: &GithubRepo,
    tag: &str,
) -> Result<PathBuf, Report> {
    let started = Instant::now();
    let spinner_style = ProgressStyle::default_spinner()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
        .template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap();

    let pb = ProgressBar::new_spinner();
    pb.set_style(spinner_style);
    pb.enable_steady_tick(Duration::from_millis(100));

    let rt = setup_tokio();

    pb.set_message(format!("Fetching release from {org}/{repo}"));

    let release_result = if tag == "latest" {
        debug!("Getting latest release from {}/{}", org, repo);
        rt.block_on(async {
            octocrab::instance()
                .repos(org, repo)
                .releases()
                .get_latest()
                .await
        })
    } else {
        debug!("Getting {} release from {}/{}", tag, org, repo);
        rt.block_on(async {
            octocrab::instance()
                .repos(org, repo)
                .releases()
                .get_by_tag(tag)
                .await
        })
    };

    let assets = match release_result {
        Ok(octocrab::models::repos::Release { assets, .. }) => assets,
        Err(err) => {
            debug!("{err:?}");
            return Err(err).wrap_err(format!(
                "Failed fetching Github release {tag:} from {org:}/{repo:}"
            ));
        }
    };

    let asset_name = asset_name(language, tag)?;
    for asset in assets.iter() {
        debug!("{:?}", &asset);
        let octocrab::models::repos::Asset { name, .. } = &asset;
        debug!("name {:?}", name);
        if *name == asset_name {
            let file = out_dir.join(repo.to_owned() + ".tar.gz");
            let mut dest = std::fs::File::create(&file)
                .wrap_err_with(|| format!("Failed to create asset download file {:?}", file))?;

            debug!(
                "Downloading asset {:?} to {:?}",
                &asset.browser_download_url, file
            );

            let target = asset.browser_download_url.clone();

            let body = reqwest::blocking::get(target).unwrap().bytes()?;
            let mut content = Cursor::new(body);

            std::io::copy(&mut content, &mut dest).unwrap();

            pb.println(format!(" {} Fetching release from {org}/{repo}", CHECKMARK));
            pb.finish_and_clear();
            println!(
                "{} fetch in {}",
                style("Finished").green().bold(),
                HumanDuration(started.elapsed())
            );

            return Ok(file);
        }
    }

    let e: Report = eyre!("Asset not found");

    Err(e).wrap_err("Github release asset download failed")
}

fn setup_tokio() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}
