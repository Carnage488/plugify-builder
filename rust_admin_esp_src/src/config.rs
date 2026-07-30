use serde::Deserialize;
use std::fs;

const CONFIG_PATH: &str = "configs/rust_admin_esp.toml";

#[derive(Debug, Clone)]
pub struct Config {
    pub admin_flag_all: String,
    pub admin_flag_death: String,
    pub ct_color: Color,
    pub t_color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub fn entity_value(self) -> String {
        format!("{} {} {} 255", self.red, self.green, self.blue)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RawConfig {
    #[serde(default = "default_admin_flag_all")]
    admin_flag_all: String,
    #[serde(default = "default_admin_flag_death")]
    admin_flag_death: String,
    #[serde(default = "default_ct_color")]
    ct_color: String,
    #[serde(default = "default_t_color")]
    t_color: String,
}

impl Default for RawConfig {
    fn default() -> Self {
        Self {
            admin_flag_all: default_admin_flag_all(),
            admin_flag_death: default_admin_flag_death(),
            ct_color: default_ct_color(),
            t_color: default_t_color(),
        }
    }
}

pub fn load() -> Result<Config, String> {
    let raw = match fs::read_to_string(CONFIG_PATH) {
        Ok(contents) => toml::from_str::<RawConfig>(&contents)
            .map_err(|error| format!("failed to parse {CONFIG_PATH}: {error}"))?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            println!("[RustAdminESP] {CONFIG_PATH} not found; using built-in defaults");
            RawConfig::default()
        }
        Err(error) => return Err(format!("failed to read {CONFIG_PATH}: {error}")),
    };

    let admin_flag_all = canonical_flags(&raw.admin_flag_all, "admin_flag_all")?;
    let admin_flag_death = canonical_flags(&raw.admin_flag_death, "admin_flag_death")?;
    let ct_color = parse_color(&raw.ct_color, "ct_color")?;
    let t_color = parse_color(&raw.t_color, "t_color")?;

    Ok(Config {
        admin_flag_all,
        admin_flag_death,
        ct_color,
        t_color,
    })
}

fn canonical_flags(value: &str, field: &str) -> Result<String, String> {
    let value = value.trim().to_ascii_lowercase();
    if value.is_empty() {
        return Err(format!("{field} cannot be empty"));
    }

    let mut chars = value.chars().collect::<Vec<_>>();
    if let Some(invalid) = chars
        .iter()
        .copied()
        .find(|character| !(('a'..='t').contains(character) || *character == 'z'))
    {
        return Err(format!(
            "{field} contains unsupported SourceMod flag '{invalid}'; allowed flags are a-t and z"
        ));
    }

    chars.sort_unstable();
    chars.dedup();
    Ok(chars.into_iter().collect())
}

fn parse_color(value: &str, field: &str) -> Result<Color, String> {
    let value = value.trim();
    let hex = value.strip_prefix('#').unwrap_or(value);
    if hex.len() != 6 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{field} must use #RRGGBB format, got '{value}'"));
    }

    let red = u8::from_str_radix(&hex[0..2], 16)
        .map_err(|error| format!("invalid {field}: {error}"))?;
    let green = u8::from_str_radix(&hex[2..4], 16)
        .map_err(|error| format!("invalid {field}: {error}"))?;
    let blue = u8::from_str_radix(&hex[4..6], 16)
        .map_err(|error| format!("invalid {field}: {error}"))?;

    Ok(Color { red, green, blue })
}

fn default_admin_flag_all() -> String {
    "z".to_string()
}

fn default_admin_flag_death() -> String {
    "b".to_string()
}

fn default_ct_color() -> String {
    "#4968F7".to_string()
}

fn default_t_color() -> String {
    "#E2AD36".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_colors() {
        assert_eq!(
            parse_color("#4968F7", "ct_color").unwrap(),
            Color {
                red: 73,
                green: 104,
                blue: 247,
            }
        );
    }

    #[test]
    fn rejects_non_sourcemod_flags() {
        assert!(canonical_flags("by", "flag").is_err());
        assert_eq!(canonical_flags("zbz", "flag").unwrap(), "bz");
    }
}
