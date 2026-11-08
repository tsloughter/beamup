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

async fn fetch_release(
    client: &octocrab::Octocrab,
    GithubRepo { org, repo }: &GithubRepo,
    tag: &str,
    tag_prefix: Option<&str>,
) -> octocrab::Result<octocrab::models::repos::Release> {
    let repository = client.repos(org, repo);
    let releases = repository.releases();
    if tag == "latest" {
        debug!("Getting latest release from {org}/{repo}");
        return releases.get_latest().await;
    }

    debug!("Getting {tag} release from {org}/{repo}");
    let result = releases.get_by_tag(tag).await;
    match (&result, tag_prefix) {
        (Err(octocrab::Error::GitHub { source, .. }), Some(prefix))
            if source.status_code == http::StatusCode::NOT_FOUND && !tag.starts_with(prefix) =>
        {
            let prefixed_tag = format!("{prefix}{tag}");
            debug!("Release {tag} not found; trying {prefixed_tag}");
            releases.get_by_tag(&prefixed_tag).await
        }
        _ => result,
    }
}

pub fn download_release_tarball(
    out_dir: &Path,
    repository: &GithubRepo,
    tag: &str,
    tag_prefix: Option<&str>,
) -> Result<PathBuf, Report> {
    let rt = setup_tokio();

    let release_result = rt.block_on(async {
        fetch_release(&octocrab::instance(), repository, tag, tag_prefix).await
    });
    let GithubRepo { org, repo } = repository;

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
    repository: &GithubRepo,
    tag: &str,
    tag_prefix: Option<&str>,
) -> Result<PathBuf, Report> {
    let rt = setup_tokio();

    let release_result = rt.block_on(async {
        fetch_release(&octocrab::instance(), repository, tag, tag_prefix).await
    });
    let GithubRepo { org, repo } = repository;

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
                asset.browser_download_url, file
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{components::Kind, installable::Installable, languages::Language};
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;

    // Exercise the real GitHub client against a local server, checking both the
    // order of tag lookups and the endpoint used for `latest`.
    fn lookup(
        tag: &str,
        prefix: Option<&str>,
        responses: &[(&str, u16)],
    ) -> octocrab::Result<octocrab::models::repos::Release> {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let base_uri = format!("http://{}", listener.local_addr().unwrap());
        let responses: Vec<_> = responses
            .iter()
            .map(|(path, status)| (path.to_string(), *status))
            .collect();
        let server = std::thread::spawn(move || {
            for (path, status) in responses {
                let deadline = Instant::now() + Duration::from_secs(5);
                let mut stream = loop {
                    match listener.accept() {
                        Ok((stream, _)) => break stream,
                        Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                            assert!(Instant::now() < deadline, "Missing request for {path}");
                            std::thread::sleep(Duration::from_millis(5));
                        }
                        Err(err) => panic!("{err}"),
                    }
                };
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut reader = BufReader::new(&stream);
                let mut request = String::new();
                reader.read_line(&mut request).unwrap();
                assert_eq!(
                    request.trim(),
                    format!("GET /repos/test/repo/releases/{path} HTTP/1.1")
                );
                loop {
                    let mut header = String::new();
                    reader.read_line(&mut header).unwrap();
                    if header == "\r\n" || header.is_empty() {
                        break;
                    }
                }
                let body = if status == 200 {
                    let tag = path.strip_prefix("tags/").unwrap_or("latest-tag");
                    format!(
                        r#"{{"url":"https://example.com/release","html_url":"https://example.com/release",
                        "assets_url":"https://example.com/assets","upload_url":"https://example.com/upload",
                        "id":1,"node_id":"release","tag_name":"{tag}","target_commitish":"main",
                        "draft":false,"prerelease":false,"assets":[]}}"#
                    )
                } else {
                    r#"{"message":"Release lookup failed"}"#.to_owned()
                };
                write!(stream, "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
            }
        });
        let result = setup_tokio().block_on(async {
            let client = octocrab::Octocrab::builder().base_uri(base_uri)?.build()?;
            fetch_release(
                &client,
                &GithubRepo {
                    org: "test".to_owned(),
                    repo: "repo".to_owned(),
                },
                tag,
                prefix,
            )
            .await
        });
        server.join().unwrap();
        result
    }

    #[test]
    fn missing_tags_try_each_languages_prefix() {
        for (language, tag, prefixed) in [
            (Language::Erlang, "28.0", "OTP-28.0"),
            (Language::Elixir, "1.19.0", "v1.19.0"),
            (Language::Gleam, "1.10.0", "v1.10.0"),
        ] {
            let release = lookup(
                tag,
                language.release_tag_prefix(),
                &[
                    (&format!("tags/{tag}"), 404),
                    (&format!("tags/{prefixed}"), 200),
                ],
            )
            .unwrap();
            assert_eq!(release.tag_name, prefixed);
        }
    }

    #[test]
    fn exact_tags_and_latest_take_precedence() {
        for tag in ["custom-tag", "v1.10.0"] {
            let release = lookup(tag, Some("v"), &[(&format!("tags/{tag}"), 200)]).unwrap();
            assert_eq!(release.tag_name, tag);
        }
        let release = lookup("latest", Some("v"), &[("latest", 200)]).unwrap();
        assert_eq!(release.tag_name, "latest-tag");
    }

    #[test]
    fn only_missing_unprefixed_language_tags_are_retried() {
        for (tag, prefix, path, status) in [
            ("1.10.0", Some("v"), "tags/1.10.0", 401),
            ("1.10.0", Some("v"), "tags/1.10.0", 403),
            ("v1.10.0", Some("v"), "tags/v1.10.0", 404),
            ("OTP-28.0", Some("OTP-"), "tags/OTP-28.0", 404),
            ("latest", Some("v"), "latest", 404),
            ("1.0", Kind::Rebar3.release_tag_prefix(), "tags/1.0", 404),
            ("1.0", Kind::Elp.release_tag_prefix(), "tags/1.0", 404),
        ] {
            let error = lookup(tag, prefix, &[(path, status)]).unwrap_err();
            assert!(
                matches!(error, octocrab::Error::GitHub { source, .. } if source.status_code.as_u16() == status)
            );
        }
        let error = lookup("1.0", Some("v"), &[("tags/1.0", 404), ("tags/v1.0", 404)]).unwrap_err();
        assert!(
            matches!(error, octocrab::Error::GitHub { source, .. } if source.status_code == http::StatusCode::NOT_FOUND)
        );
    }
}
