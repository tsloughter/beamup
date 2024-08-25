use color_eyre::eyre::Result;
use std::env::Args;

use crate::config;
use std::path::*;
use std::process::Command;

pub fn run(bin: &str, args: Args) -> Result<()> {
    // no -c argument available in this case
    let dir = config::install_to_use(bin)?;
    let cmd = Path::new(&dir).join("bin").join(bin);

    debug!("running {:?}", cmd);

    let mut binding = Command::new(cmd);
    let cmd = binding.args(args);

    exec(cmd)
}

#[cfg(unix)]
fn exec(cmd: &mut Command) -> Result<()> {
    use std::os::unix::prelude::*;
    Err(cmd.exec().into())
}

// thanks rustup command.rs
#[cfg(windows)]
fn exec(cmd: &mut Command) -> Result<()> {
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

    let status = cmd.status()?;
    if !status.success() {
        std::process::exit(status.code().unwrap_or(0))
    }

    Ok(())
}
