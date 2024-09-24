use crate::config;
use crate::languages;
use crate::links;
use crate::components;
use color_eyre::eyre::Result;

pub fn run(maybe_language: Option<&languages::Language>, config: &config::Config) -> Result<()> {
    let bin_dir = config::bin_dir();
    let _ = std::fs::create_dir_all(&bin_dir);
    let b = languages::bins(config);

    if let Some(language) = maybe_language {
        let bins = b
            .iter()
            .filter_map(|(a, b)| if *b == *language { Some(a) } else { None });
        links::update(bins, &bin_dir)
    } else {
        let bins = b.iter().map(|(a, _)| a);
        links::update(bins, &bin_dir)?;

        // also update component links
        let bins = components::bins().into_iter().map(|(a, _)| a);
        links::update(bins, &bin_dir)
    }
}
