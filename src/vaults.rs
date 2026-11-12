use std::env;
use std::path::{Path, PathBuf};

use crate::config;
use crate::i18n::t;

/// Load the vault list from the config file.
/// A vault given on the command line is added to the list (saved to the config file if new)
/// and becomes the current vault. Without an argument, the first existing vault is current.
/// Returns the list and the index of the current vault.
pub fn startup(arg: Option<&String>) -> Result<(Vec<PathBuf>, usize), String> {
    let path = config::config_path().ok_or_else(|| t("err.no_config_dir", &[]))?;
    let mut cfg = config::load(&path)?;
    let current = match arg {
        Some(input) => {
            let dir = absolute(input)?;
            match position(&cfg.vaults, &dir) {
                Some(index) => index,
                None => {
                    cfg.vaults.push(dir);
                    config::save(&path, &cfg)?;
                    cfg.vaults.len() - 1
                }
            }
        }
        None => cfg
            .vaults
            .iter()
            .position(|v| v.is_dir())
            .ok_or_else(|| t("err.no_vault", &[("path", &path.display().to_string())]))?,
    };
    Ok((cfg.vaults, current))
}

/// Make a user-typed folder path absolute (relative to the current directory) and check it exists.
pub fn absolute(input: &str) -> Result<PathBuf, String> {
    let path = Path::new(input);
    let full = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir().map_err(|e| e.to_string())?.join(path)
    };
    if full.is_dir() {
        Ok(full)
    } else {
        Err(t("err.not_folder", &[("input", input)]))
    }
}

/// Index of `dir` in `list`, comparing canonical paths so different spellings match.
pub fn position(list: &[PathBuf], dir: &Path) -> Option<usize> {
    let real = dir.canonicalize().ok();
    list.iter()
        .position(|v| v == dir || (real.is_some() && v.canonicalize().ok() == real))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_matches_same_folder_by_canonical_path() {
        let dir = std::env::temp_dir();
        let list = vec![PathBuf::from("Z:/does/not/exist"), dir.clone()];
        assert_eq!(position(&list, &dir), Some(1));
        assert_eq!(position(&list, &dir.join("missing")), None);
    }

    #[test]
    fn absolute_rejects_missing_folder() {
        assert!(absolute("Z:/does/not/exist").is_err());
    }
}
