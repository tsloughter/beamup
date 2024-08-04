use crate::github::GithubRepo;

pub fn get_github_repo() -> GithubRepo {
    GithubRepo {
        org: "gleam-lang".to_string(),
        repo: "gleam".to_string(),
    }
}
