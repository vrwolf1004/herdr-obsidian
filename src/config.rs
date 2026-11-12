use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::i18n::t;

/// Config file contents. `vaults` is the list shown in the vault panel, in order.
#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct Config {
    #[serde(default)]
    pub vaults: Vec<PathBuf>,
    /// Screen language ("en", "ko", ...). Empty means auto-detect. See i18n.rs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// Location of the config file: %APPDATA% on Windows, $XDG_CONFIG_HOME or ~/.config elsewhere.
pub fn config_path() -> Option<PathBuf> {
    let base = if cfg!(windows) {
        PathBuf::from(env::var_os("APPDATA")?)
    } else if let Some(xdg) = env::var_os("XDG_CONFIG_HOME") {
        PathBuf::from(xdg)
    } else {
        PathBuf::from(env::var_os("HOME")?).join(".config")
    };
    Some(base.join("herdr-obsidian").join("config.toml"))
}

/// A missing file is an empty config, not an error.
pub fn load(path: &Path) -> Result<Config, String> {
    match fs::read_to_string(path) {
        Ok(text) => toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display())),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(Config::default()),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

/// Rewrites the whole file. Comments and unknown keys in an existing file are not kept.
pub fn save(path: &Path, cfg: &Config) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let text =
        toml::to_string(cfg).map_err(|e| t("err.config_encode", &[("error", &e.to_string())]))?;
    fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_vaults_list() {
        let cfg: Config = toml::from_str("vaults = ['C:/notes', 'D:/work']").unwrap();
        assert_eq!(
            cfg.vaults,
            [PathBuf::from("C:/notes"), PathBuf::from("D:/work")]
        );
    }

    #[test]
    fn empty_config_has_no_vaults() {
        let cfg: Config = toml::from_str("").unwrap();
        assert!(cfg.vaults.is_empty());
        assert_eq!(cfg.lang, None);
    }

    #[test]
    fn lang_is_read_and_kept_when_unset_it_is_not_written() {
        let cfg: Config = toml::from_str("lang = 'ko'").unwrap();
        assert_eq!(cfg.lang.as_deref(), Some("ko"));
        let unset = Config::default();
        assert!(!toml::to_string(&unset).unwrap().contains("lang"));
    }

    #[test]
    fn save_then_load_roundtrip_keeps_windows_paths() {
        let dir = std::env::temp_dir().join(format!("ho-config-{}", std::process::id()));
        let path = dir.join("config.toml");
        let cfg = Config {
            vaults: vec![PathBuf::from(r"C:\work\obsidian\herdr_test")],
            lang: Some("ko".to_string()),
        };
        save(&path, &cfg).unwrap();
        assert_eq!(load(&path).unwrap(), cfg);
        let _ = fs::remove_dir_all(&dir);
    }
}
