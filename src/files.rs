use std::fs::{self, OpenOptions};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::i18n::t;

/// Check a file or folder name typed by the user (trimmed). Rejects names that would
/// escape the folder or that Windows forbids.
pub fn check_name(input: &str) -> Result<String, String> {
    let name = input.trim();
    if name.is_empty() {
        return Err(t("err.name_empty", &[]));
    }
    if name.starts_with('.')
        || name
            .chars()
            .any(|c| c.is_control() || "/\\:*?\"<>|".contains(c))
    {
        return Err(t("err.name_invalid", &[("name", name)]));
    }
    Ok(name.to_string())
}

/// Like `check_name`, and append `.md` when missing.
pub fn note_name(input: &str) -> Result<String, String> {
    let name = check_name(input)?;
    if name.to_lowercase().ends_with(".md") {
        Ok(name)
    } else {
        Ok(format!("{name}.md"))
    }
}

/// Create a folder. Fails if the name already exists (never reuses an existing folder).
pub fn create_folder(parent: &Path, name: &str) -> Result<PathBuf, String> {
    let path = parent.join(name);
    match fs::create_dir(&path) {
        Ok(()) => Ok(path),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => Err(t("err.exists", &[("name", name)])),
        Err(e) => Err(t("err.folder_create", &[("error", &e.to_string())])),
    }
}

/// Create an empty note. Never overwrites: `create_new` fails if the name exists.
pub fn create_note(dir: &Path, name: &str) -> Result<PathBuf, String> {
    let path = dir.join(name);
    match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(_) => Ok(path),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => Err(t("err.exists", &[("name", name)])),
        Err(e) => Err(t("err.note_create", &[("error", &e.to_string())])),
    }
}

/// Rename a note in place. Refuses to replace another file.
/// A case-only change (a.md -> A.md) is allowed even though the target "exists" on Windows.
pub fn rename_note(from: &Path, new_name: &str) -> Result<PathBuf, String> {
    let to = from.with_file_name(new_name);
    let case_only = from
        .file_name()
        .is_some_and(|old| old.to_string_lossy().eq_ignore_ascii_case(new_name));
    if to.exists() && !case_only {
        return Err(t("err.exists", &[("name", new_name)]));
    }
    fs::rename(from, &to).map_err(|e| t("err.rename", &[("error", &e.to_string())]))?;
    Ok(to)
}

/// Move a note to `<root>/.trash/` (no restore UI; recover from the folder by hand).
/// Keeps the name. On a clash it becomes `name.<UTC time>.md`, with `-2`, `-3`... if needed.
/// The trash sits under the list root, so the rename stays on one volume.
pub fn trash_note(root: &Path, path: &Path, now: SystemTime) -> Result<PathBuf, String> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or_else(|| t("err.no_name", &[]))?;
    let dir = root.join(".trash");
    fs::create_dir_all(&dir).map_err(|e| t("err.trash_dir", &[("error", &e.to_string())]))?;
    let mut target = dir.join(&name);
    if target.exists() {
        let (stem, ext) = match name.rsplit_once('.') {
            Some((stem, ext)) => (stem.to_string(), format!(".{ext}")),
            None => (name.clone(), String::new()),
        };
        let stamp = utc_stamp(now);
        target = dir.join(format!("{stem}.{stamp}{ext}"));
        let mut n = 2;
        while target.exists() {
            target = dir.join(format!("{stem}.{stamp}-{n}{ext}"));
            n += 1;
        }
    }
    fs::rename(path, &target).map_err(|e| t("err.trash_move", &[("error", &e.to_string())]))?;
    Ok(target)
}

/// `YYYYMMDD-HHMMSS` in UTC. Done by hand so no timezone crate is needed.
fn utc_stamp(now: SystemTime) -> String {
    let secs = now
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as i64;
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    // Civil date from a day count (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}{m:02}{d:02}-{:02}{:02}{:02}",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ho-files-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn note_name_adds_md_and_rejects_bad_names() {
        assert_eq!(note_name("  memo ").unwrap(), "memo.md");
        assert_eq!(note_name("memo.MD").unwrap(), "memo.MD");
        assert!(note_name("").is_err());
        assert!(note_name("..").is_err());
        assert!(note_name(".hidden").is_err());
        assert!(note_name("a/b").is_err());
        assert!(note_name("a\\b").is_err());
        assert!(note_name("a:b").is_err());
    }

    #[test]
    fn create_note_never_overwrites() {
        let dir = temp_dir("create");
        let path = create_note(&dir, "a.md").unwrap();
        fs::write(&path, "keep me").unwrap();
        assert_eq!(
            create_note(&dir, "a.md").unwrap_err(),
            t("err.exists", &[("name", "a.md")])
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), "keep me");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rename_refuses_existing_target() {
        let dir = temp_dir("rename");
        let a = create_note(&dir, "a.md").unwrap();
        create_note(&dir, "b.md").unwrap();
        assert_eq!(
            rename_note(&a, "b.md").unwrap_err(),
            t("err.exists", &[("name", "b.md")])
        );
        assert!(a.exists());
        let moved = rename_note(&a, "c.md").unwrap();
        assert_eq!(moved, dir.join("c.md"));
        assert!(!a.exists() && moved.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn create_folder_refuses_existing_name() {
        let dir = temp_dir("folder");
        let made = create_folder(&dir, "Inbox").unwrap();
        assert!(made.is_dir());
        assert!(create_folder(&dir, "Inbox")
            .unwrap_err()
            .eq(&t("err.exists", &[("name", "Inbox")])));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn utc_stamp_known_dates() {
        assert_eq!(utc_stamp(UNIX_EPOCH), "19700101-000000");
        // 2000-02-29 00:00:00 UTC (leap day)
        let leap = UNIX_EPOCH + std::time::Duration::from_secs(951_782_400);
        assert_eq!(utc_stamp(leap), "20000229-000000");
        let late = UNIX_EPOCH + std::time::Duration::from_secs(951_782_400 + 86_399);
        assert_eq!(utc_stamp(late), "20000229-235959");
    }

    #[test]
    fn trash_moves_note_and_renames_on_clash() {
        let root = temp_dir("trash");
        let now = UNIX_EPOCH + std::time::Duration::from_secs(951_782_400);
        let first = create_note(&root, "a.md").unwrap();
        let moved = trash_note(&root, &first, now).unwrap();
        assert_eq!(moved, root.join(".trash").join("a.md"));
        assert!(!first.exists() && moved.exists());

        // Same name again: keep both, add the time before .md.
        let second = create_note(&root, "a.md").unwrap();
        let moved2 = trash_note(&root, &second, now).unwrap();
        assert_eq!(moved2, root.join(".trash").join("a.20000229-000000.md"));
        assert!(moved.exists() && moved2.exists());
        let _ = fs::remove_dir_all(&root);
    }
}
