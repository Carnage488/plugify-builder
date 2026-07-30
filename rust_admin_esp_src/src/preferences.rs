use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

const PREFERENCES_PATH: &str = "configs/rust_admin_esp_users.toml";
const TEMP_PREFERENCES_PATH: &str = "configs/rust_admin_esp_users.toml.tmp";

static ENABLED_STEAM_IDS: OnceLock<Mutex<HashSet<u64>>> = OnceLock::new();

#[derive(Debug, Default, Serialize, Deserialize)]
struct PreferenceFile {
    #[serde(default)]
    enabled_steam_ids: Vec<u64>,
}

pub fn initialize() -> Result<usize, String> {
    let enabled = load_preferences()?;
    let count = enabled.len();
    ENABLED_STEAM_IDS
        .set(Mutex::new(enabled))
        .map_err(|_| "RustAdminESP preferences were already initialized".to_string())?;
    Ok(count)
}

pub fn is_enabled(steam_id: u64) -> bool {
    if steam_id == 0 {
        return false;
    }

    let Some(state) = ENABLED_STEAM_IDS.get() else {
        return false;
    };
    state
        .lock()
        .map(|enabled| enabled.contains(&steam_id))
        .unwrap_or(false)
}

pub fn set_enabled(steam_id: u64, should_enable: bool) -> Result<bool, String> {
    if steam_id == 0 {
        return Err("cannot persist ESP for SteamID64 0".to_string());
    }

    let state = ENABLED_STEAM_IDS
        .get()
        .ok_or_else(|| "RustAdminESP preferences are not initialized".to_string())?;
    let mut enabled = state
        .lock()
        .map_err(|_| "RustAdminESP preferences lock is poisoned".to_string())?;

    let changed = if should_enable {
        enabled.insert(steam_id)
    } else {
        enabled.remove(&steam_id)
    };
    if !changed {
        return Ok(false);
    }

    if let Err(error) = save_preferences(&enabled) {
        if should_enable {
            enabled.remove(&steam_id);
        } else {
            enabled.insert(steam_id);
        }
        return Err(error);
    }

    Ok(true)
}

fn load_preferences() -> Result<HashSet<u64>, String> {
    let contents = match fs::read_to_string(PREFERENCES_PATH) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(HashSet::new());
        }
        Err(error) => {
            return Err(format!(
                "failed to read {PREFERENCES_PATH}: {error}"
            ));
        }
    };

    let parsed = toml::from_str::<PreferenceFile>(&contents)
        .map_err(|error| format!("failed to parse {PREFERENCES_PATH}: {error}"))?;
    Ok(parsed
        .enabled_steam_ids
        .into_iter()
        .filter(|steam_id| *steam_id != 0)
        .collect())
}

fn save_preferences(enabled: &HashSet<u64>) -> Result<(), String> {
    if let Some(parent) = Path::new(PREFERENCES_PATH).parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!("failed to create {}: {error}", parent.display())
        })?;
    }

    let mut enabled_steam_ids = enabled.iter().copied().collect::<Vec<_>>();
    enabled_steam_ids.sort_unstable();
    let contents = toml::to_string_pretty(&PreferenceFile {
        enabled_steam_ids,
    })
    .map_err(|error| format!("failed to serialize ESP preferences: {error}"))?;

    let write_result = (|| -> Result<(), std::io::Error> {
        let mut file = File::create(TEMP_PREFERENCES_PATH)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
        fs::rename(TEMP_PREFERENCES_PATH, PREFERENCES_PATH)?;
        Ok(())
    })();

    if let Err(error) = write_result {
        let _ = fs::remove_file(TEMP_PREFERENCES_PATH);
        return Err(format!(
            "failed to atomically save {PREFERENCES_PATH}: {error}"
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preference_file_serializes_sorted_ids() {
        let enabled = HashSet::from([76561198000000002_u64, 76561198000000001_u64]);
        let mut ids = enabled.iter().copied().collect::<Vec<_>>();
        ids.sort_unstable();
        let serialized = toml::to_string(&PreferenceFile {
            enabled_steam_ids: ids,
        })
        .unwrap();
        assert!(serialized.contains("76561198000000001"));
        assert!(serialized.contains("76561198000000002"));
    }
}
