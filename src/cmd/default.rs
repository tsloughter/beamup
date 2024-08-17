use crate::config;
use crate::languages::Language;
use color_eyre::eyre::Result;

pub fn run(
    language: &Language,
    id: &String,
    config_file: String,
    config: config::Config,
) -> Result<()> {
    config::set_default(language, id, config_file, config)
}
