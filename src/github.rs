use crate::languages;
use std::io::Cursor;

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

pub fn download_asset(language: &languages::Language, tag: &str) {
    let rt = setup_tokio();

    let (org, repo) = languages::get_github_org_repo(language);
    let octocrab::models::repos::Release { assets, .. } = rt.block_on(async {
        octocrab::instance()
            .repos(org, repo)
            .releases()
            .get_by_tag(tag)
            .await
            .unwrap()
    });
    // release = get_by_tag
    //     release = get_latest

    let suffix = constants_to_gleam_suffix().unwrap();
    for asset in assets.iter() {
        let octocrab::models::repos::Asset { name, .. } = asset;
        debug!("name {:?}", name);
        if name.ends_with(suffix) {
            debug!("asset {:?}", asset.browser_download_url);
            let target = asset.browser_download_url.clone();

            let body = reqwest::blocking::get(target).unwrap().bytes().unwrap();
            let mut content = Cursor::new(body);
            let mut dest = std::fs::File::create(repo.to_owned() + ".tar.gz").unwrap();
            std::io::copy(&mut content, &mut dest).unwrap();
            break;
        }
    }
}

fn setup_tokio() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}
