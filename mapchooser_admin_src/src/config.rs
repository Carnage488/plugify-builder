use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::sync::{OnceLock, RwLock};

pub const CONFIG_PATH: &str = "configs/mapchooser/admin.toml";
const DEFAULT_CONFIG_TOML: &str = include_str!("../configs/mapchooser/admin.toml");

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AdminSettings {
    pub permission: String,
    pub default_flags: String,
    pub menu_order: i32,
    pub vote_duration_seconds: f64,
    pub vote_warning_seconds: u32,
    pub max_vote_maps: usize,
    pub items_per_page: usize,
    pub change_delay_seconds: f32,
}

impl Default for AdminSettings {
    fn default() -> Self {
        Self {
            permission: "mapchooser.admin.maps".to_string(),
            default_flags: "g".to_string(),
            menu_order: 50,
            vote_duration_seconds: 30.0,
            vote_warning_seconds: 5,
            max_vote_maps: 5,
            items_per_page: 5,
            change_delay_seconds: 2.0,
        }
    }
}

fn storage() -> &'static RwLock<AdminSettings> {
    static SETTINGS: OnceLock<RwLock<AdminSettings>> = OnceLock::new();
    SETTINGS.get_or_init(|| RwLock::new(AdminSettings::default()))
}

pub fn load_from_disk() -> Result<AdminSettings, String> {
    ensure_file(CONFIG_PATH, DEFAULT_CONFIG_TOML)?;
    let raw = fs::read_to_string(CONFIG_PATH)
        .map_err(|error| format!("cannot read '{CONFIG_PATH}': {error}"))?;
    let mut value = toml::from_str::<toml::Value>(&raw)
        .map_err(|error| format!("cannot parse '{CONFIG_PATH}': {error}"))?;
    // v0.2.6 compatibility: silently discard the removed warning-sound setting.
    if let Some(table) = value.as_table_mut() {
        table.remove("vote_warning_sound");
    }
    let parsed: AdminSettings = value
        .try_into()
        .map_err(|error| format!("cannot parse '{CONFIG_PATH}': {error}"))?;
    validate(parsed)
}

fn ensure_file(path: &str, contents: &str) -> Result<(), String> {
    let target = Path::new(path);
    if target.is_file() {
        return Ok(());
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "cannot create config directory '{}': {error}",
                parent.display()
            )
        })?;
    }
    fs::write(target, contents)
        .map_err(|error| format!("cannot create default config '{}': {error}", target.display()))
}

fn validate(mut settings: AdminSettings) -> Result<AdminSettings, String> {
    settings.permission = settings.permission.trim().to_string();
    settings.default_flags = normalize_default_flags(&settings.default_flags)?;

    if settings.permission.is_empty() {
        return Err("admin.permission cannot be empty".to_string());
    }
    if settings.permission.len() > 128 {
        return Err("admin.permission cannot exceed 128 characters".to_string());
    }
    if !(-10_000..=10_000).contains(&settings.menu_order) {
        return Err("admin.menu_order must be within -10000..=10000".to_string());
    }
    if !(5.0..=120.0).contains(&settings.vote_duration_seconds) {
        return Err("admin.vote_duration_seconds must be within 5..=120".to_string());
    }
    if settings.vote_warning_seconds > 30 {
        return Err("admin.vote_warning_seconds must be within 0..=30".to_string());
    }
    if !(1..=5).contains(&settings.max_vote_maps) {
        return Err("admin.max_vote_maps must be within 1..=5".to_string());
    }
    if !(1..=5).contains(&settings.items_per_page) {
        return Err("admin.items_per_page must be within 1..=5".to_string());
    }
    if !(0.0..=30.0).contains(&settings.change_delay_seconds) {
        return Err("admin.change_delay_seconds must be within 0..=30".to_string());
    }

    Ok(settings)
}

fn normalize_default_flags(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(
            "admin.default_flags cannot be empty because RustAdmin treats an empty mask as public access"
                .to_string(),
        );
    }
    if trimmed.len() > 128 {
        return Err("admin.default_flags cannot exceed 128 characters".to_string());
    }

    let mut flags = Vec::new();
    for flag in trimmed.chars() {
        if !flag.is_ascii_lowercase() || !(('a'..='t').contains(&flag) || flag == 'z') {
            return Err(
                "admin.default_flags must be a compact SourceMod-style flag string using a-t and z, for example 'g' or 'bg'"
                    .to_string(),
            );
        }
        if !flags.contains(&flag) {
            flags.push(flag);
        }
    }
    flags.sort_unstable();
    Ok(flags.into_iter().collect())
}

pub fn replace(settings: AdminSettings) {
    match storage().write() {
        Ok(mut guard) => *guard = settings,
        Err(poisoned) => *poisoned.into_inner() = settings,
    }
}

pub fn snapshot() -> AdminSettings {
    match storage().read() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_default_config_is_valid() {
        let parsed = toml::from_str::<AdminSettings>(DEFAULT_CONFIG_TOML)
            .expect("default admin.toml must parse");
        validate(parsed).expect("default admin.toml must pass validation");
    }

    #[test]
    fn unknown_config_fields_are_rejected() {
        let raw = format!("{}\nmax_vote_map = 3\n", DEFAULT_CONFIG_TOML);
        toml::from_str::<AdminSettings>(&raw)
            .expect_err("unknown fields must not be silently ignored");
    }

    #[test]
    fn fallback_flags_are_normalized_and_validated() {
        assert_eq!(normalize_default_flags(" gbg ").unwrap(), "bg");
        assert_eq!(normalize_default_flags("zgba").unwrap(), "abgz");
        assert!(normalize_default_flags("   ").is_err());
        assert!(normalize_default_flags("b,g").is_err());
        assert!(normalize_default_flags("u").is_err());
        assert!(normalize_default_flags("B").is_err());
    }

    #[test]
    fn removed_warning_sound_field_is_accepted_for_migration() {
        let raw = format!(
            "{}\nvote_warning_sound = \"sounds/ui/counter_beep.vsnd\"\n",
            DEFAULT_CONFIG_TOML
        );
        let mut value = toml::from_str::<toml::Value>(&raw).expect("legacy config must parse");
        value
            .as_table_mut()
            .expect("admin config must be a table")
            .remove("vote_warning_sound");
        let _: AdminSettings = value
            .try_into()
            .expect("legacy sound field must be ignored during migration");
    }
}
