use crate::github::print_releases;
use crate::languages::LanguageStruct;

pub fn run(language: &LanguageStruct) {
    print_releases(&language.source_repo);
}
