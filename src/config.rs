use anyhow::{Context, Result, anyhow, bail};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::str::FromStr;

const APP_DIR: &str = "tody";
const CONFIG_FILENAME: &str = "config.toml";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DefaultView {
    /// Auto-detect: project-local when in a git repo, global otherwise
    #[default]
    Auto,
    Merged,
    Local,
    Global,
}

impl Display for DefaultView {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Merged => write!(f, "merged"),
            Self::Local => write!(f, "local"),
            Self::Global => write!(f, "global"),
        }
    }
}

impl FromStr for DefaultView {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "merged" => Ok(Self::Merged),
            "local" => Ok(Self::Local),
            "global" => Ok(Self::Global),
            _ => bail!("invalid default_view '{value}', expected auto|merged|local|global"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub default_view: DefaultView,
    #[serde(default = "default_color_local")]
    pub color_local: String,
    #[serde(default = "default_color_global")]
    pub color_global: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default_view: DefaultView::Auto,
            color_local: default_color_local(),
            color_global: default_color_global(),
        }
    }
}

impl AppConfig {
    pub fn load_or_default() -> Result<Self> {
        let path = config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("failed reading config {}", path.display()))?;
        let parsed = toml::from_str::<Self>(&raw)
            .with_context(|| format!("failed parsing config {}", path.display()))?;
        Ok(parsed)
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("failed creating config directory {}", parent.display())
            })?;
        }
        let raw = toml::to_string_pretty(self).context("failed serializing config to TOML")?;
        std::fs::write(&path, raw)
            .with_context(|| format!("failed writing config {}", path.display()))?;
        Ok(())
    }

    pub fn set_key(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "default_view" => self.default_view = value.parse()?,
            "color_local" => self.color_local = normalize_color_value(value),
            "color_global" => self.color_global = normalize_color_value(value),
            _ => bail!("unsupported config key '{key}'"),
        }
        Ok(())
    }

    pub fn get_key(&self, key: &str) -> Result<String> {
        match key {
            "default_view" => Ok(self.default_view.to_string()),
            "color_local" => Ok(self.color_local.clone()),
            "color_global" => Ok(self.color_global.clone()),
            _ => bail!("unsupported config key '{key}'"),
        }
    }

    pub fn keys() -> [&'static str; 3] {
        ["default_view", "color_local", "color_global"]
    }
}

pub fn config_path() -> Result<PathBuf> {
    let base = config_dir().ok_or_else(|| anyhow!("unable to resolve config directory"))?;
    Ok(base.join(APP_DIR).join(CONFIG_FILENAME))
}

fn normalize_color_value(raw: &str) -> String {
    raw.trim().to_ascii_lowercase()
}

fn default_color_local() -> String {
    "bright_magenta".to_string()
}

fn default_color_global() -> String {
    "bright_cyan".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_view_values() -> Result<()> {
        assert_eq!("auto".parse::<DefaultView>()?, DefaultView::Auto);
        assert_eq!("merged".parse::<DefaultView>()?, DefaultView::Merged);
        assert_eq!("local".parse::<DefaultView>()?, DefaultView::Local);
        assert_eq!("global".parse::<DefaultView>()?, DefaultView::Global);
        Ok(())
    }

    #[test]
    fn rejects_invalid_default_view() {
        assert!("other".parse::<DefaultView>().is_err());
    }

    #[test]
    fn set_and_get_keys_round_trip() -> Result<()> {
        let mut cfg = AppConfig::default();
        cfg.set_key("default_view", "local")?;
        cfg.set_key("color_local", "blue")?;
        cfg.set_key("color_global", "yellow")?;

        assert_eq!(cfg.get_key("default_view")?, "local");
        assert_eq!(cfg.get_key("color_local")?, "blue");
        assert_eq!(cfg.get_key("color_global")?, "yellow");
        Ok(())
    }

    #[test]
    fn rejects_unsupported_key() {
        let mut cfg = AppConfig::default();
        assert!(cfg.set_key("unknown_key", "value").is_err());
        assert!(cfg.get_key("unknown_key").is_err());
    }

    #[test]
    fn default_config_values() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.default_view, DefaultView::Auto);
        assert_eq!(cfg.color_local, "bright_magenta");
        assert_eq!(cfg.color_global, "bright_cyan");
    }
}
