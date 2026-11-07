#![cfg(target_os = "linux")]

use std::{
    fs,
    io::Write,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, Output, Stdio},
};
use tempdir::TempDir;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

struct Fixture {
    root: TempDir,
    install_dir: PathBuf,
    config: PathBuf,
    project: PathBuf,
}

impl Fixture {
    fn new() -> Result<Self> {
        let root = TempDir::new("beamup-filesystem-cli")?;
        let install_dir = root.path().join("custom install root");
        let config = root.path().join("config/beamup/config.toml");
        let project = root.path().join("project");
        fs::create_dir_all(config.parent().unwrap())?;
        fs::create_dir_all(&project)?;
        fs::write(&config, format!("install_dir = {:?}\n", install_dir))?;
        Ok(Self {
            root,
            install_dir,
            config,
            project,
        })
    }

    fn command(&self, program: impl AsRef<std::ffi::OsStr>) -> Command {
        let mut command = Command::new(program);
        command
            .current_dir(&self.project)
            .env("XDG_CONFIG_HOME", self.root.path().join("config"))
            .env("XDG_DATA_HOME", self.root.path().join("unrelated-data"))
            .env_remove("BEAMUP_OTP_MAJOR_VSN")
            .env("NO_COLOR", "1");
        command
    }

    fn cli(&self, args: &[&str]) -> Result<Output> {
        Ok(self
            .command(env!("CARGO_BIN_EXE_beamup"))
            .arg("--config")
            .arg(&self.config)
            .args(args)
            .output()?)
    }

    fn install(&self, tool: &str, id: &str, bin: &str) -> Result {
        let dir = self.install_dir.join(tool).join(id);
        fs::create_dir_all(dir.join("bin"))?;
        let executable = dir.join("bin").join(bin);
        fs::write(
            &executable,
            format!("#!/bin/sh\nprintf '%s\\n' '{tool}:{id}' \"$@\"\n"),
        )?;
        fs::set_permissions(executable, fs::Permissions::from_mode(0o755))?;
        fs::write(
            dir.join(".beamup-install.toml"),
            "release = 'underlying-release'\n",
        )?;
        Ok(())
    }

    fn shim(&self, bin: &str) -> Result<PathBuf> {
        let path = self.root.path().join(bin);
        std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_beamup"), &path)?;
        Ok(path)
    }
}

fn stdout(output: Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn list_discovers_sorted_languages_and_components_without_a_registry() -> Result {
    let f = Fixture::new()?;
    f.install("erlang", "z-debug", "erl")?;
    f.install("erlang", "a-release", "erl")?;
    f.install("gleam", "custom-name", "gleam")?;
    f.install("elp", "editor", "elp")?;
    f.install("rebar3", "build-tool", "rebar3")?;
    fs::create_dir_all(f.install_dir.join("erlang/incomplete"))?;
    fs::create_dir_all(f.install_dir.join("erlang/.beamup-install-abandoned/bin"))?;
    fs::write(f.install_dir.join("erlang/not-a-directory"), "file")?;
    // A stale registry must neither add phantom installs nor override paths.
    let mut config = fs::read_to_string(&f.config)?;
    config.push_str("\n[erlang.installs.phantom]\ndir = '/missing'\nrelease = 'old'\n");
    fs::write(&f.config, config)?;
    assert_eq!(stdout(f.cli(&["list"])?), "Elixir:\n\nErlang:\na-release\nz-debug\n\nGleam:\ncustom-name\n\nELP:\neditor\n\nrebar3:\nbuild-tool\n");
    Ok(())
}

#[test]
fn default_and_switch_validate_filesystem_installs_and_only_save_preferences() -> Result {
    let f = Fixture::new()?;
    f.install("gleam", "debug", "gleam")?;
    stdout(f.cli(&["default", "gleam", "debug"])?);
    let text = fs::read_to_string(&f.config)?;
    let config: toml::Value = toml::from_str(&text)?;
    assert_eq!(config["gleam"]["default"].as_str(), Some("debug"));
    assert!(config["gleam"].get("installs").is_none());
    assert!(!f.cli(&["default", "gleam", "missing"])?.status.success());
    assert_eq!(fs::read_to_string(&f.config)?, text);
    stdout(f.cli(&["switch", "gleam", "debug"])?);
    let local = fs::read_to_string(f.project.join(".beamup.toml"))?;
    assert_eq!(local, "gleam = \"debug\"\n");
    assert!(!f.cli(&["switch", "gleam", "missing"])?.status.success());
    assert_eq!(fs::read_to_string(f.project.join(".beamup.toml"))?, local);
    assert_eq!(
        fs::read_to_string(f.install_dir.join("gleam/debug/.beamup-install.toml"))?,
        "release = 'underlying-release'\n"
    );
    Ok(())
}

#[test]
fn shims_use_configured_root_and_local_ids_for_languages_and_components() -> Result {
    let f = Fixture::new()?;
    for (tool, bin) in [
        ("gleam", "gleam"),
        ("erlang", "erl"),
        ("rebar3", "rebar3"),
        ("elp", "elp"),
    ] {
        f.install(tool, "global", bin)?;
        f.install(tool, "project", bin)?;
    }
    let mut config = fs::read_to_string(&f.config)?;
    for tool in ["gleam", "erlang", "rebar3", "elp"] {
        config.push_str(&format!("\n[{tool}]\ndefault = 'global'\n"));
    }
    fs::write(&f.config, config)?;
    for (tool, bin) in [
        ("gleam", "gleam"),
        ("erlang", "erl"),
        ("rebar3", "rebar3"),
        ("elp", "elp"),
    ] {
        let shim = f.shim(bin)?;
        assert_eq!(
            stdout(f.command(&shim).arg("argument with spaces").output()?),
            format!("{tool}:global\nargument with spaces\n")
        );
        fs::write(
            f.project.join(".beamup.toml"),
            format!("{tool} = 'project'\n"),
        )?;
        assert_eq!(
            stdout(f.command(&shim).output()?),
            format!("{tool}:project\n")
        );
        fs::write(
            f.project.join(".beamup.toml"),
            format!("{tool} = '../escape'\n"),
        )?;
        let output = f.command(&shim).output()?;
        assert!(!output.status.success());
        assert!(String::from_utf8(output.stderr)?.contains("Invalid install ID"));
        fs::remove_file(f.project.join(".beamup.toml"))?;
    }
    Ok(())
}

#[test]
fn shims_start_executables_by_name_so_erlang_wrappers_find_erl() -> Result {
    // erlc, escript and friends locate `erl` from their argv[0] and split it on
    // whitespace. The fixture's install root contains a space, so this only
    // passes when the shim starts the executable by bare name through PATH,
    // with this install's bin directory first. A script cannot observe argv[0]
    // (the kernel hands the interpreter the resolved path instead), so the fake
    // erlc is the shell itself, reading its commands from stdin.
    let f = Fixture::new()?;
    f.install("erlang", "wrapper", "erl")?;
    let bin_dir = f.install_dir.join("erlang/wrapper/bin");
    std::os::unix::fs::symlink("/bin/sh", bin_dir.join("erlc"))?;
    stdout(f.cli(&["default", "erlang", "wrapper"])?);
    let shim = f.shim("erlc")?;
    let mut child = f
        .command(shim)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"echo \"$0\"\ncommand -v erl\n")?;
    assert_eq!(
        stdout(child.wait_with_output()?),
        format!("erlc\n{}\n", bin_dir.join("erl").display())
    );
    Ok(())
}

#[test]
fn relative_install_root_is_resolved_against_the_config_directory() -> Result {
    let f = Fixture::new()?;
    // Include parent components to exercise normalization for staged source builds.
    fs::write(
        &f.config,
        "install_dir = '../../temporary/../custom install root'\n",
    )?;
    f.install("gleam", "relative", "gleam")?;
    stdout(f.cli(&["default", "gleam", "relative"])?);
    let shim = f.shim("gleam")?;
    assert_eq!(stdout(f.command(shim).output()?), "gleam:relative\n");
    Ok(())
}

#[test]
fn invalid_ids_fail_before_downloading_or_writing() -> Result {
    let f = Fixture::new()?;
    let original_config = fs::read(&f.config)?;
    for id in [
        "",
        ".",
        "..",
        "../escape",
        "/tmp/escape",
        "nested/id",
        "nested\\id",
        "C:escape",
        ".beamup-install-test",
    ] {
        for args in [
            vec!["install", "gleam", "v1", "--id", id],
            vec!["component", "install", "rebar3", "v1", "--id", id],
            vec!["build", "erlang", "OTP-28", "--id", id],
            vec!["default", "gleam", id],
            vec!["switch", "gleam", id],
        ] {
            let output = f.cli(&args)?;
            assert!(!output.status.success(), "{args:?}");
            assert!(
                String::from_utf8(output.stderr)?.contains("Invalid install ID"),
                "{args:?}"
            );
        }
    }
    assert_eq!(fs::read(&f.config)?, original_config);
    assert!(!f.install_dir.exists());
    assert!(!f.project.join(".beamup.toml").exists());
    Ok(())
}
