use crate::config::Config;
use color_eyre::eyre::{eyre, Result, WrapErr};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::ErrorKind,
    path::{Component, Path, PathBuf},
};
use tempdir::TempDir;

pub const METADATA_FILE: &str = ".beamup-install.toml";
const STAGING_PREFIX: &str = ".beamup-install-";

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Metadata {
    pub release: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub otp_vsn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_options: Option<String>,
}

pub fn validate_id(id: &str) -> Result<()> {
    let mut components = Path::new(id).components();
    if id.is_empty()
        || id == "."
        || id == ".."
        || id.starts_with(STAGING_PREFIX)
        || id
            .chars()
            .any(|c| c.is_control() || "<>:\"/\\|?*".contains(c))
        || id.ends_with(['.', ' '])
        || !matches!(components.next(), Some(Component::Normal(_)))
        || components.next().is_some()
    {
        return Err(eyre!("Invalid install ID {id:?}: expected a single directory name (the {STAGING_PREFIX} prefix is reserved)"));
    }
    Ok(())
}

pub fn path(config: &Config, tool: &str, id: &str) -> Result<PathBuf> {
    validate_id(id)?;
    Ok(config.install_dir.join(tool).join(id))
}

pub fn lookup(config: &Config, tool: &str, id: &str) -> Result<PathBuf> {
    let dir = path(config, tool, id)?;
    let valid = match fs::metadata(dir.join("bin")) {
        Ok(metadata) => metadata.is_dir(),
        Err(error) if matches!(error.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) => {
            false
        }
        Err(error) => {
            return Err(error)
                .wrap_err_with(|| format!("Unable to inspect install {}", dir.display()))
        }
    };
    if !valid || incomplete_marker(&dir).try_exists()? {
        return Err(eyre!(
            "No complete install found for {tool} ID {id:?} at {}",
            dir.display()
        ));
    }
    Ok(dir)
}

pub fn discover(config: &Config, tool: &str) -> Result<Vec<String>> {
    let dir = config.install_dir.join(tool);
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => {
            return Err(e).wrap_err_with(|| format!("Unable to list installs in {}", dir.display()))
        }
    };
    let mut ids = Vec::new();
    for entry in entries {
        let entry = entry?;
        if let Some(id) = entry.file_name().to_str() {
            if validate_id(id).is_ok()
                && entry.path().join("bin").is_dir()
                && !incomplete_marker(&entry.path()).try_exists()?
            {
                ids.push(id.to_owned());
            }
        }
    }
    ids.sort();
    Ok(ids)
}

/// Work in the destination filesystem so publishing requires only a rename.
/// Dropping an unsuccessful install removes its staging directory.
pub struct PendingInstall {
    staging: TempDir,
    destination: PathBuf,
    force: bool,
}

impl PendingInstall {
    pub fn new(destination: &Path, force: bool) -> Result<Self> {
        check_destination(destination, force)?;
        let parent = destination
            .parent()
            .ok_or_else(|| eyre!("Install has no parent directory"))?;
        fs::create_dir_all(parent)?;
        Ok(Self {
            staging: TempDir::new_in(parent, STAGING_PREFIX)?,
            destination: destination.to_owned(),
            force,
        })
    }

    pub fn staging_dir(&self) -> &Path {
        self.staging.path()
    }

    pub fn payload(&self) -> PathBuf {
        self.staging.path().join("install")
    }

    pub fn publish(self, payload: &Path, metadata: &Metadata) -> Result<()> {
        validate_payload(payload)?;
        fs::write(payload.join(METADATA_FILE), toml::to_string(metadata)?)?;
        self.replace(false, |destination| {
            fs::rename(payload, destination).wrap_err("Failed publishing install")
        })
    }

    // Native installers may register their destination in the OS. Run them at
    // the final path, hiding incomplete output and retaining the old install
    // until the installer and its initialization have both succeeded.
    #[cfg(any(windows, test))]
    pub fn install_in_place(
        self,
        metadata: &Metadata,
        install: impl FnOnce(&Path) -> Result<()>,
    ) -> Result<()> {
        self.replace(true, |destination| {
            // Keep the marker beside the install: native installers can tighten
            // destination ACLs. Likewise, write metadata before running them.
            fs::write(incomplete_marker(destination), b"")?;
            fs::create_dir_all(destination)?;
            fs::write(destination.join(METADATA_FILE), toml::to_string(metadata)?)?;
            install(destination)?;
            validate_payload(destination)?;
            fs::remove_file(incomplete_marker(destination))?;
            Ok(())
        })
    }

    fn replace(
        self,
        cleanup_partial: bool,
        install: impl FnOnce(&Path) -> Result<()>,
    ) -> Result<()> {
        check_destination(&self.destination, self.force)?;
        let previous = self.staging.path().join("previous");
        // Keep the previous install until its replacement is fully extracted/built.
        let had_previous = match fs::symlink_metadata(&self.destination) {
            Ok(_) => {
                fs::rename(&self.destination, &previous)?;
                true
            }
            Err(e) if e.kind() == ErrorKind::NotFound => false,
            Err(e) => return Err(e.into()),
        };
        if let Err(error) = install(&self.destination) {
            if cleanup_partial {
                let cleanup = match fs::symlink_metadata(&self.destination) {
                    Ok(metadata) if metadata.is_dir() => fs::remove_dir_all(&self.destination),
                    Ok(_) => fs::remove_file(&self.destination),
                    Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
                    Err(e) => Err(e),
                };
                if let Err(cleanup_error) = cleanup {
                    let saved = self.staging.into_path();
                    return Err(eyre!("Install failed: {error}; unable to remove incomplete output: {cleanup_error}. Recovery files retained at {}", saved.display()));
                }
            }
            if had_previous {
                if let Err(restore_error) = fs::rename(&previous, &self.destination) {
                    // Preserve the backup if rollback fails; don't delete it on drop.
                    let saved = self.staging.into_path();
                    return Err(eyre!("Install failed: {error}; failed restoring previous install: {restore_error}. Backup retained at {}", saved.join("previous").display()));
                }
            }
            if cleanup_partial {
                match fs::remove_file(incomplete_marker(&self.destination)) {
                    Ok(()) => {}
                    Err(e) if e.kind() == ErrorKind::NotFound => {}
                    Err(e) => {
                        return Err(error)
                            .wrap_err(format!("Unable to clear incomplete marker: {e}"))
                    }
                }
            }
            return Err(error);
        }
        Ok(())
    }
}

pub fn check_destination(destination: &Path, force: bool) -> Result<()> {
    match fs::symlink_metadata(destination) {
        Ok(_) if !force => Err(eyre!(
            "Install directory already exists. Use `-f` to replace {}",
            destination.display()
        )),
        Ok(_) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

fn incomplete_marker(destination: &Path) -> PathBuf {
    destination.with_file_name(format!(
        "{STAGING_PREFIX}{}.incomplete",
        destination.file_name().unwrap().to_string_lossy()
    ))
}

fn validate_payload(payload: &Path) -> Result<()> {
    if !payload.join("bin").is_dir() || fs::read_dir(payload.join("bin"))?.next().is_none() {
        return Err(eyre!(
            "Install did not produce a populated bin directory at {}",
            payload.display()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_installer_uses_final_path_and_hides_output_until_complete() -> Result<()> {
        let root = TempDir::new("beamup-native-installer")?;
        let config: Config = toml::from_str(&format!("install_dir = {:?}", root.path()))?;
        let destination = path(&config, "erlang", "test")?;
        let pending = PendingInstall::new(&destination, false)?;
        pending.install_in_place(
            &Metadata {
                release: "OTP-28".to_owned(),
                ..Metadata::default()
            },
            |dir| {
                assert_eq!(dir, destination);
                fs::create_dir_all(dir.join("bin"))?;
                fs::write(dir.join("bin/erl.exe"), b"installed")?;
                assert!(discover(&config, "erlang")?.is_empty());
                assert!(lookup(&config, "erlang", "test").is_err());
                Ok(())
            },
        )?;
        assert_eq!(discover(&config, "erlang")?, ["test"]);
        assert_eq!(lookup(&config, "erlang", "test")?, destination);
        assert!(destination.join(METADATA_FILE).is_file());
        Ok(())
    }

    #[test]
    fn native_installer_failure_restores_previous_files_and_metadata() -> Result<()> {
        let root = TempDir::new("beamup-native-installer-failure")?;
        let destination = root.path().join("erlang/test");
        fs::create_dir_all(destination.join("bin"))?;
        fs::write(destination.join("bin/erl.exe"), b"previous")?;
        fs::write(destination.join(METADATA_FILE), "release = 'old'\n")?;
        let pending = PendingInstall::new(&destination, true)?;
        let result = pending.install_in_place(&Metadata::default(), |dir| {
            fs::create_dir_all(dir.join("bin"))?;
            fs::write(dir.join("bin/erl.exe"), b"partial replacement")?;
            Err(eyre!("installer failed"))
        });
        assert!(result.is_err());
        assert_eq!(fs::read(destination.join("bin/erl.exe"))?, b"previous");
        assert_eq!(
            fs::read_to_string(destination.join(METADATA_FILE))?,
            "release = 'old'\n"
        );
        assert!(!incomplete_marker(&destination).exists());
        assert_eq!(fs::read_dir(destination.parent().unwrap())?.count(), 1);
        Ok(())
    }
}
