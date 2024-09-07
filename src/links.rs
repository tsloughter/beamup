use color_eyre::{eyre::eyre, eyre::Result};
use std::path::Path;
use std::path::PathBuf;

pub fn update<S>(bins: impl Iterator<Item = S>, bin_dir: &PathBuf) -> Result<()>
where
    S: AsRef<str> + AsRef<Path>,
{
    let mut has_err = false;
    for b in bins {
        let link = Path::new(&bin_dir).join(b);
        let beamup_exe = std::env::current_exe().unwrap();
        debug!("linking {:?} to {:?}", link, beamup_exe);
        let _ = std::fs::remove_file(&link);
        match std::fs::hard_link(&beamup_exe, &link) {
            Ok(()) => {}
            Err(e) => {
                has_err = true;
                error!("Failed to link {:?} to {:?}: {}", link, beamup_exe, e);
            }
        }
    }

    // TODO: should do a multi-report error instead of this
    if has_err {
        Err(eyre!("Some links failed to be created"))
    } else {
        Ok(())
    }
}
