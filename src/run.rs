use std::env::Args;
use std::os::unix::prelude::CommandExt;
use std::path::*;
use std::process::Command;

use crate::config;

pub fn run(bin: &str, args: Args) {
    // no -c argument available in this case
    let dir = config::install_to_use();
    let cmd = Path::new(&dir).join("bin").join(bin);

    debug!("running {}", cmd.to_str().unwrap());

    let _ = Command::new(cmd.to_str().unwrap()).args(args).exec();
}
