use crate::languages;
use std::path::Path;

pub fn run(maybe_language: Option<&languages::Language>) {
    for (b, l) in languages::BIN_MAP.iter() {
        if maybe_language.is_none() || maybe_language.is_some_and(|x| *x == *l) {
            let home_dir = dirs::home_dir().unwrap(); //config::lookup_cache_dir(&config);
            let bin_dir = Path::new(&home_dir).join(".local").join("bin");
            let _ = std::fs::create_dir_all(&bin_dir);

            let link = Path::new(&bin_dir).join(b);
            let beamup_exe = std::env::current_exe().unwrap();
            debug!("linking {:?} to {:?}", link, beamup_exe);
            match std::fs::remove_file(&link) {
                Ok(()) => match std::fs::hard_link(beamup_exe, &link) {
                    Ok(()) => {}
                    Err(e) => {
                        error!("Failed to symlink {:?}: {}", link, e);
                    }
                },
                Err(e) => {
                    error!("Failed to remove symlink {:?}: {}", link, e);
                }
            }
        }
    }
}
