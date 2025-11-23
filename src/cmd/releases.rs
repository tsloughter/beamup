use crate::github::print_releases;
use crate::languages;

pub fn run(installable: Box<dyn languages::Installable>) {
    // TODO: source repo and binary repo could have different releases to print
    print_releases(&installable.source_repo());
}
