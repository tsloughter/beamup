use crate::components;
use crate::config;
use color_eyre::eyre::eyre;
use color_eyre::eyre::Result;
use std::env;
use std::env::{join_paths, split_paths, Args};
use std::path::*;
use std::process::Command;

pub fn run_component(
    bin: &str,
    kind: &components::Kind,
    args: Args,
    config: &config::Config,
) -> Result<()> {
    run_install(&config::component_install_to_use(kind, config)?, bin, args)
}

pub fn run(bin: &str, args: Args, config: &config::Config) -> Result<()> {
    run_install(&config::install_to_use_by_bin(bin, config)?, bin, args)
}

fn run_install(dir: &Path, bin: &str, args: Args) -> Result<()> {
    let bin_dir = dir.join("bin");
    let executable = bin_dir.join(bin);
    #[cfg(windows)]
    let executable = if executable.extension().is_none() {
        executable.with_extension("exe")
    } else {
        executable
    };
    if !executable.is_file() {
        return Err(eyre!(
            "Executable not found in selected install: {}",
            executable.display()
        ));
    }
    let path = env::var_os("PATH").unwrap_or_default();
    let mut paths = split_paths(&path).collect::<Vec<_>>();
    paths.insert(0, bin_dir);
    // Start the executable by name through the prepended PATH, not by its full
    // path. Erlang's erlc, escript and similar wrappers derive the `erl` to run
    // from their own argv[0] and split that value on whitespace, so an absolute
    // argv[0] under a directory like "~/Library/Application Support" fails.
    // A bare name makes them fall back to `erl` on PATH, which is this install's.
    let mut command = Command::new(bin);
    command.env("PATH", join_paths(paths)?).args(args);
    debug!("Running {:?}", command);
    exec(&mut command)
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
