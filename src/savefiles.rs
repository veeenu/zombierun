use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use anyhow::{anyhow, Result};

use crate::Game;

pub struct Savefile {
    pub data: Vec<u8>,
    pub saved: Instant,
    pub uid: usize,
}

impl Savefile {
    pub fn new(path: &Path) -> Result<Self> {
        static UID: AtomicUsize = AtomicUsize::new(0);
        let saved = Instant::now();
        let data = std::fs::read(path)?;
        let uid = UID.fetch_add(1, Ordering::Relaxed);

        Ok(Self { data, saved, uid })
    }

    pub fn saved(&self) -> &Instant {
        &self.saved
    }

    pub fn load(&self, path: &Path) -> Result<()> {
        std::fs::write(path, &self.data)?;
        Ok(())
    }
}

pub struct SavefilePath {
    pub game: Game,
    pub path: PathBuf,
}

impl SavefilePath {
    pub fn get_all() -> Result<Vec<SavefilePath>> {
        let er = get_savefiles_path_eldenring()?
            .into_iter()
            .map(|path| SavefilePath {
                game: Game::EldenRing,
                path,
            });

        let ds3 = get_savefiles_path_ds3()?
            .into_iter()
            .map(|path| SavefilePath {
                game: Game::DarkSoulsIII,
                path,
            });

        Ok(ds3.chain(er).collect())
    }
}

impl Display for SavefilePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: ", self.game)?;
        let path = self.path.to_string_lossy();
        if path.len() > 32 {
            write!(f, "...{}", &path[path.len() - 32..])?;
            
        } else {
            write!(f, "{path}")?;
        }

        Ok(())
    }
}

fn get_savefiles_path_eldenring() -> Result<Vec<PathBuf>> {
    let re = regex::Regex::new(r"^[a-f0-9]+$").unwrap();
    let savefile_path: PathBuf = [
        std::env::var("APPDATA")
            .map_err(|e| anyhow!("{}", e))?
            .as_str(),
        "EldenRing",
    ]
    .iter()
    .collect();

    Ok(std::fs::read_dir(savefile_path)
        .map_err(|e| anyhow!("{}", e))?
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            if re.is_match(&e.file_name().to_string_lossy()) && e.path().is_dir() {
                Some(e.path())
            } else {
                None
            }
        })
        .map(|path| path.join("ER0000.sl2"))
        .collect())
}

fn get_savefiles_path_ds3() -> Result<Vec<PathBuf>> {
    let re = regex::Regex::new(r"^[a-f0-9]+$").unwrap();
    let savefile_path: PathBuf = [
        std::env::var("APPDATA")
            .map_err(|e| anyhow!("{}", e))?
            .as_str(),
        "DarkSoulsIII",
    ]
    .iter()
    .collect();

    Ok(std::fs::read_dir(savefile_path)
        .map_err(|e| anyhow!("{}", e))?
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            if re.is_match(&e.file_name().to_string_lossy()) && e.path().is_dir() {
                Some(e.path())
            } else {
                None
            }
        })
        .map(|path| path.join("DS30000.sl2"))
        .collect())
}
