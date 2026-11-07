use crate::{components::Kind, config, installs, languages::Language};
use color_eyre::eyre::Result;
use strum::IntoEnumIterator;

pub fn run(config: &config::Config) -> Result<()> {
    let tools = Language::iter()
        .map(|language| (language.label(), language.to_string()))
        .chain(Kind::iter().map(|kind| (kind.label(), kind.to_string())));
    for (index, (heading, tool)) in tools.enumerate() {
        if index > 0 {
            println!();
        }
        println!("{heading}:");
        for id in installs::discover(config, &tool)? {
            println!("{id}");
        }
    }
    Ok(())
}
