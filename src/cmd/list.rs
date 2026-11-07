use crate::{config, installs};
use color_eyre::eyre::Result;

pub fn run(config: &config::Config) -> Result<()> {
    for (index, (heading, tool)) in [
        ("Elixir", "elixir"),
        ("Erlang", "erlang"),
        ("Gleam", "gleam"),
        ("ELP", "elp"),
        ("rebar3", "rebar3"),
    ]
    .iter()
    .enumerate()
    {
        if index > 0 {
            println!();
        }
        println!("{heading}:");
        for id in installs::discover(config, tool)? {
            println!("{id}");
        }
    }
    Ok(())
}
