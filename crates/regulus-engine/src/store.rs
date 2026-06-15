//! Persistence of profiles + the active selection. Default config dir is
//! %APPDATA%\regulus\config.toml; load_from/save_to take an explicit path for
//! tests.

use crate::profile::Profile;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredState {
    pub active: String,
    pub profiles: Vec<Profile>,
}

impl Default for StoredState {
    fn default() -> Self {
        StoredState {
            active: "Balanced".into(),
            profiles: Profile::presets(),
        }
    }
}

pub fn config_path() -> PathBuf {
    let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
    PathBuf::from(base).join("regulus").join("config.toml")
}

pub fn load_from(path: &Path) -> Result<StoredState, String> {
    match std::fs::read_to_string(path) {
        Ok(s) => toml::from_str(&s).map_err(|e| e.to_string()),
        Err(_) => Ok(StoredState::default()),
    }
}

pub fn save_to(path: &Path, state: &StoredState) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let s = toml::to_string_pretty(state).map_err(|e| e.to_string())?;
    std::fs::write(path, s).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::Profile;

    #[test]
    fn save_then_load_roundtrips_active_and_profiles() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let state = StoredState {
            active: "Balanced".into(),
            profiles: Profile::presets(),
        };
        save_to(&path, &state).unwrap();
        let loaded = load_from(&path).unwrap();
        assert_eq!(loaded.active, "Balanced");
        assert_eq!(loaded.profiles.len(), 3);
    }

    #[test]
    fn load_missing_file_returns_default_seeded_state() {
        let dir = tempfile::tempdir().unwrap();
        let loaded = load_from(&dir.path().join("nope.toml")).unwrap();
        assert_eq!(loaded.active, "Balanced");
        assert_eq!(loaded.profiles.len(), 3);
    }
}
