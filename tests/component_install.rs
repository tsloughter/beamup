// Linux's XDG directories let these CLI tests isolate all application state.
#![cfg(target_os = "linux")]

use std::fs;
use std::process::Command;
use tempdir::TempDir;

#[test]
fn component_install_checks_the_selected_id_directory() -> Result<(), Box<dyn std::error::Error>> {
    let root = TempDir::new("beamup-component-cli")?;
    let data_dir = root.path().join("data");
    let config_file = root.path().join("config.toml");

    let install_root = root.path().join("custom-installs");
    fs::write(&config_file, format!("install_dir = {:?}\n", install_root))?;
    let release = "same-release";

    for component in ["elp", "rebar3"] {
        let component_dir = install_root.join(component);
        // Every candidate directory already exists, so the install must fail before
        // downloading, and the error must name the selected ID, not the release version.
        for id in [release, "first", "second"] {
            let install_dir = component_dir.join(id);
            fs::create_dir_all(&install_dir)?;
            fs::write(install_dir.join("keep"), id)?;
        }

        for id in [Some("first"), Some("second"), None] {
            let mut command = Command::new(env!("CARGO_BIN_EXE_beamup"));
            command
                .current_dir(root.path())
                .env("XDG_DATA_HOME", &data_dir)
                .arg("--config")
                .arg(&config_file)
                .args(["component", "install", component, release]);
            if let Some(id) = id {
                command.args(["--id", id]);
            }
            let output = command.output()?;
            let stderr = String::from_utf8(output.stderr)?;
            assert!(!output.status.success(), "{component}: {stderr}");
            assert!(
                stderr.contains("Install directory already exists"),
                "{stderr}"
            );
            let expected_dir = component_dir.join(id.unwrap_or(release));
            assert!(stderr.contains(expected_dir.to_str().unwrap()), "{stderr}");
            assert_eq!(
                fs::read_to_string(expected_dir.join("keep"))?,
                id.unwrap_or(release)
            );
        }
    }
    Ok(())
}
