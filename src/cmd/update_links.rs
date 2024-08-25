use crate::config;
use crate::languages;
use color_eyre::{eyre::eyre, eyre::Result};
use std::path::Path;

pub fn run(maybe_language: Option<&languages::Language>) -> Result<()> {
    let mut has_err = false;
    for (b, l) in languages::BIN_MAP.iter() {
        if maybe_language.is_none() || maybe_language.is_some_and(|x| *x == *l) {
            let bin_dir = config::bin_dir();
            let _ = std::fs::create_dir_all(&bin_dir);

            let link = Path::new(&bin_dir).join(b);
            let beamup_exe = std::env::current_exe().unwrap();
            debug!("linking {:?} to {:?}", link, beamup_exe);
            let _ = std::fs::remove_file(&link);
            match std::fs::hard_link(beamup_exe, &link) {
                Ok(()) => {}
                Err(e) => {
                    has_err = true;
                    error!("Failed to link {:?}: {}", link, e);
                }
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
