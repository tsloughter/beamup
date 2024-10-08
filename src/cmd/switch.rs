use crate::config;
use crate::languages::Language;
use color_eyre::eyre::Result;

pub fn run(language: &Language, id: &str, config: config::Config) -> Result<()> {
    config::switch(language, id, &config)
}
