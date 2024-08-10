use crate::github::GithubRelease;
// use http::Uri;

pub enum GitRef {
    Branch(String),
    Release(GithubRelease),
}

impl std::fmt::Display for GitRef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GitRef::Branch(b) => write!(f, "{}", b),
            GitRef::Release(r) => write!(f, "{}", r),
        }
    }
}

// pub struct GitInfo {
//     repo: Uri,
// }
