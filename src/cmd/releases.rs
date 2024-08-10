use crate::github::print_releases;
use crate::languages::{get_github_repo, Language};

pub fn run(language: &Language) {
    let github_repo = get_github_repo(language);
    print_releases(&github_repo);
}
