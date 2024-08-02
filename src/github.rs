use crate::languages;
use console::{style, Emoji};
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use std::io::Cursor;
use std::time::Duration;
use std::time::Instant;

// http://unicode.org/emoji/charts/full-emoji-list.html
static CHECKMARK: Emoji = Emoji("✅", "✅ ");
static FAIL: Emoji = Emoji("❌", "❌ ");
static WARNING: Emoji = Emoji("🚫", "🚫");

fn constants_to_gleam_suffix() -> Result<&'static str, &'static str> {
    match (std::env::consts::ARCH, std::env::consts::OS) {
        ("x86_64", "linux") => Ok("x86_64-unknown-linux-musl.tar.gz"),
        ("aarch", "linux") => Ok("aarch-unknown-linux-musl.tar.gz"),
        ("x86_64", "apple") => Ok("x86_64-apple-darwin.tar.gz"),
        ("aarch", "apple") => Ok("aarch-apple-darwin.tar.gz"),
        ("x86_64", "windows") => Ok("x86_64-pc-windows-msvc.zip"),
        _ => Err("an unsupported architecture and operating system combo"),
    }
}

pub fn print_releases(language: &languages::Language) {
    let rt = setup_tokio();
    let (org, repo) = languages::get_github_org_repo(language);

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

pub fn download_asset(language: &languages::Language, tag: &str) -> Option<String> {
    let started = Instant::now();
    let spinner_style = ProgressStyle::default_spinner()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
        .template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap();

    let pb = ProgressBar::new_spinner();
    pb.set_style(spinner_style);
    pb.enable_steady_tick(Duration::from_millis(100));

    let rt = setup_tokio();

    let (org, repo) = languages::get_github_org_repo(language);

    pb.set_message(format!("Fetching release from {org}/{repo}"));

    let octocrab::models::repos::Release { assets, .. } = if tag == "latest" {
        debug!("Getting latest release from {}/{}", org, repo);
        rt.block_on(async {
            octocrab::instance()
                .repos(org, repo)
                .releases()
                .get_latest()
                .await
                .unwrap()
        })
    } else {
        debug!("Getting {} release from {}/{}", tag, org, repo);
        rt.block_on(async {
            octocrab::instance()
                .repos(org, repo)
                .releases()
                .get_by_tag(tag)
                .await
                .unwrap()
        })
    };

    let suffix = constants_to_gleam_suffix().unwrap();
    for asset in assets.iter() {
        let octocrab::models::repos::Asset { name, .. } = asset;
        debug!("name {:?}", name);
        if name.ends_with(suffix) {
            debug!("asset {:?}", asset.browser_download_url);
            let target = asset.browser_download_url.clone();

            let body = reqwest::blocking::get(target).unwrap().bytes().unwrap();
            let mut content = Cursor::new(body);
            let file = repo.to_owned() + ".tar.gz";
            let mut dest = std::fs::File::create(file.clone()).unwrap();
            std::io::copy(&mut content, &mut dest).unwrap();
            return Some(file.to_string());
        }
    }

    pb.println(format!(" {} Fetching release from {org}/{repo}", CHECKMARK));
    pb.finish_and_clear();
    println!(
        "{} fetch in {}",
        style("Finished").green().bold(),
        HumanDuration(started.elapsed())
    );

    return None;
}

fn setup_tokio() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}
