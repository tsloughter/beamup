use color_eyre::{eyre::eyre, eyre::Report, eyre::Result, eyre::WrapErr};
use console::{style, Emoji};
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

// http://unicode.org/emoji/charts/full-emoji-list.html
static CHECKMARK: Emoji = Emoji("✅", "✅ ");
// static FAIL: Emoji = Emoji("❌", "❌ ");
// static WARNING: Emoji = Emoji("🚫", "🚫");

pub type GithubRelease = String;

pub struct GithubRepo {
    pub org: String,
    pub repo: String,
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

pub fn download_release_tarball(
    out_dir: &Path,
    GithubRepo { org, repo }: &GithubRepo,
    tag: &String,
) -> Result<PathBuf, Report> {
    let rt = setup_tokio();

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

    let url = match release_result {
        Ok(octocrab::models::repos::Release {
            tarball_url: Some(tarball_url),
            ..
        }) => tarball_url,
        Ok(octocrab::models::repos::Release {
            tarball_url: None, ..
        }) => {
            let e: Report = eyre!("no source tarball found for release {tag}");
            return Err(e);
        }
        Err(err) => {
            debug!("{err:?}");
            return Err(err).wrap_err(format!(
                "Failed downloading release tarball Github release {tag:} from {org:}/{repo:}"
            ));
        }
    };

    let file = out_dir.join(repo.to_owned() + ".tar.gz");
    let dest = std::fs::File::create(&file)
        .wrap_err_with(|| format!("Failed to create asset download file {:?}", file))?;

    debug!(
        "Downloading release source tarball {:?} to {:?}",
        url.as_str(),
        file
    );

    http_download(
        dest,
        url.as_str(),
        format!("Downloading release source tarball from {org}/{repo}"),
    )?;

    Ok(file)
}

pub fn download_asset(
    asset_prefix: &regex::Regex,
    out_dir: &Path,
    GithubRepo { org, repo }: &GithubRepo,
    tag: &str,
) -> Result<PathBuf, Report> {
    let rt = setup_tokio();

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

    debug!("looking for asset {asset_prefix}");
    match assets
        .iter()
        .find(|&asset| asset_prefix.is_match(&asset.name))
    {
        Some(asset) => {
            let file = out_dir.join(&asset.name);
            let dest = std::fs::File::create(&file)
                .wrap_err_with(|| format!("Failed to create asset download file {:?}", file))?;

            debug!(
                "Downloading release asset {:?} to {:?}",
                &asset.browser_download_url, file
            );

            http_download(
                dest,
                asset.browser_download_url.as_str(),
                format!("Downloading release source tarball from {org}/{repo}"),
            )?;

            Ok(file)
        }
        None => {
            let e: Report =
                eyre!("Release found but no asset on release matching prefix {asset_prefix} found");

            Err(e).wrap_err("Github release asset download failed")
        }
    }
}

fn http_download(mut dest: File, url: &str, progress_msg: String) -> Result<()> {
    let started = Instant::now();
    let response = ureq::get(url).call()?;

    if let Some(length) = response
        .header("content-length")
        .and_then(|l| l.parse().ok())
    {
        let bar = indicatif::ProgressBar::new(!0)
            .with_prefix("Downloading")
            .with_style(
                indicatif::ProgressStyle::default_spinner()
                    .template("{prefix:>12.bright.cyan} {spinner} {msg:.cyan}")
                    .unwrap(),
            )
            .with_message("preparing");

        bar.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{prefix:>12.bright.cyan} [{bar:27}] {bytes:>9}/{total_bytes:9}  {bytes_per_sec} {elapsed:>4}/{eta:4} - {msg:.cyan}")?.progress_chars("=> "));
        bar.set_length(length);

        std::io::copy(&mut bar.wrap_read(response.into_reader()), &mut dest)?;

        bar.finish_and_clear();
    } else {
        let spinner_style = ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{prefix:.bold.dim} {spinner} {wide_msg}")
            .unwrap();

        let pb = ProgressBar::new_spinner();
        pb.set_style(spinner_style);
        pb.enable_steady_tick(Duration::from_millis(100));

        pb.set_message(progress_msg.clone());

        std::io::copy(&mut response.into_reader(), &mut dest)?;

        pb.println(format!(" {} {}", CHECKMARK, progress_msg));

        pb.finish_and_clear();

        println!(
            "{} download in {}",
            style("Finished").green().bold(),
            HumanDuration(started.elapsed())
        );
    }

    Ok(())
}

// just need this for ocotocrab
fn setup_tokio() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}
