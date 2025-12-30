use crate::github::print_releases;
use crate::installable::Installable;

pub fn run<T: Installable>(installable: &T) {
    // TODO: source repo and binary repo could have different releases to print
    print_releases(&installable.source_repo());
}
