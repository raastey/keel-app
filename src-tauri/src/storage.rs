use crate::{domain::AppData, error::Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct Store {
    directory: PathBuf,
    file: PathBuf,
}

impl Store {
    pub fn open(directory: PathBuf) -> Result<(Self, AppData)> {
        fs::create_dir_all(&directory)?;
        let file = directory.join("keel-data.json");
        let store = Self { directory, file };
        let data = if store.file.exists() {
            serde_json::from_slice(&fs::read(&store.file)?)?
        } else {
            AppData::default()
        };
        Ok((store, data))
    }

    pub fn save(&self, data: &AppData) -> Result<()> {
        let bytes = serde_json::to_vec_pretty(data)?;
        let temporary = self.directory.join("keel-data.json.next");
        fs::write(&temporary, bytes)?;
        if self.file.exists() {
            let backup = self.directory.join("keel-data.json.backup");
            fs::copy(&self.file, backup)?;
        }
        fs::rename(temporary, &self.file)?;
        Ok(())
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_creates_backup() {
        let temp = tempfile::tempdir().unwrap();
        let (store, mut data) = Store::open(temp.path().to_path_buf()).unwrap();
        data.onboarding_complete = true;
        store.save(&data).unwrap();
        store.save(&data).unwrap();
        let (_, loaded) = Store::open(temp.path().to_path_buf()).unwrap();
        assert!(loaded.onboarding_complete);
        assert!(temp.path().join("keel-data.json.backup").exists());
    }
}
