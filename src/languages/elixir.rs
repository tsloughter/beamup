use crate::github::GithubRepo;

pub fn get_github_repo() -> GithubRepo {
    GithubRepo {
        org: "elixir-lang".to_string(),
        repo: "elixir".to_string(),
    }
}
