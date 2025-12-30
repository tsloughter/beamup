use crate::components;
use crate::config;
use crate::languages;
use crate::links;
use color_eyre::eyre::Result;

pub fn run(maybe_language: Option<&languages::Language>, config: &config::Config) -> Result<()> {
    let bin_dir = config::bin_dir();
    let _ = std::fs::create_dir_all(&bin_dir);
    let b = languages::bins(config);

    let _ = if let Some(language) = maybe_language {
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
    };

    print_instructions(bin_dir)
}

#[cfg(unix)]
fn print_instructions(bin_dir: std::path::PathBuf) -> Result<()> {
    let dir_string = bin_dir.into_os_string().into_string().unwrap();
    info!(
        "\nEnsure PATH contains directory {}\n\nFor example add to your shell rc file:\n\n    export PATH={}:$PATH\n",
        dir_string, dir_string
    );

    Ok(())
}

#[cfg(windows)]
fn print_instructions(bin_dir: std::path::PathBuf) -> Result<()> {
    let dir_string = bin_dir.into_os_string().into_string().unwrap();
    info!(
        "\nEnsure PATH contains directory {}\n\nFor example:\n\n    setx PATH \"%PATH%;{}\"\n",
        dir_string, dir_string
    );

    Ok(())
}
