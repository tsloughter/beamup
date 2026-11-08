use std::{fs, process::Command};
use tempdir::TempDir;

#[test]
fn downloads_initialize_the_github_client_inside_tokio() -> Result<(), Box<dyn std::error::Error>> {
    let root = TempDir::new("beamup-release-cli")?;
    let config_file = root.path().join("config.toml");
    let install_root = root.path().join("installs");
    fs::write(&config_file, format!("install_dir = {:?}\n", install_root))?;

    // Each command gets a fresh process so the global client is uninitialized.
    // A control character makes the release URI invalid, exercising client
    // startup without sending any network requests. A separate valid ID lets
    // the command reach the download path before failing.
    for (command, language) in [("install", "gleam"), ("build", "erlang")] {
        let output = Command::new(env!("CARGO_BIN_EXE_beamup"))
            .current_dir(root.path())
            .arg("--config")
            .arg(&config_file)
            .args([command, language, "invalid\nrelease", "--id", "test"])
            .env("NO_COLOR", "1")
            .output()?;
        let stderr = String::from_utf8(output.stderr)?;
        assert_eq!(output.status.code(), Some(1), "{command}: {stderr}");
        assert!(
            stderr.contains("invalid uri character"),
            "{command}: {stderr}"
        );
    }
    Ok(())
}
