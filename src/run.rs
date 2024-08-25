use color_eyre::{eyre::Report, eyre::Result};
use std::env::Args;

use std::path::*;
use std::process::{Command, ExitStatus};

use crate::config;

pub fn run(bin: &str, args: Args) -> Result<(), Report> {
    // no -c argument available in this case
    let dir = config::install_to_use(bin)?;
    let cmd = Path::new(&dir).join("bin").join(bin);

    debug!("running {:?}", cmd);

    let mut binding = Command::new(cmd);
    let cmd = binding.args(args);

    exec(cmd)
}

#[cfg(unix)]
fn exec(cmd: &mut Command) -> Result<(), Report> {
    use std::os::unix::prelude::*;
    Err(cmd.exec().into())
}

// thanks rustup command.rs
#[cfg(windows)]
fn exec(cmd: &mut Command) -> Result<ExitStatus, Report> {
    use color_eyre::eyre::eyre;
    use windows_sys::Win32::Foundation::{BOOL, FALSE, TRUE};
    use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;

    unsafe extern "system" fn ctrlc_handler(_: u32) -> BOOL {
        // Do nothing. Let the child process handle it.
        TRUE
    }
    unsafe {
        if SetConsoleCtrlHandler(Some(ctrlc_handler), TRUE) == FALSE {
            return Err(eyre!("Unable to set console handler",));
        }
    }

    Ok(cmd.status()?)
}
