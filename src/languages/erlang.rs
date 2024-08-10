use crate::github::GithubRepo;

pub fn get_github_repo() -> GithubRepo {
    GithubRepo {
        org: "erlang".to_string(),
        repo: "otp".to_string(),
    }
}
