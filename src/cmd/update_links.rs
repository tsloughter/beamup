use crate::config;
use crate::languages;
use crate::links;
use color_eyre::eyre::Result;

pub fn run(maybe_language: Option<&languages::Language>) -> Result<()> {
    let bin_dir = config::bin_dir();
    let _ = std::fs::create_dir_all(&bin_dir);

    let bins: Vec<_> = if let Some(language) = maybe_language {
        languages::BIN_MAP
            .iter()
            .filter_map(|(a, b)| if *b == *language { Some(a) } else { None })
            .collect()
    } else {
        languages::BIN_MAP.iter().map(|(a, _)| a).collect()
    };

    links::update(bins.into_iter(), &bin_dir)
}
