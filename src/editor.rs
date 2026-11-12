use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};

fn modified_time(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// Write to a temp file in the same folder, sync it, then rename over `path`.
/// A crash or failed write leaves the old note intact. The temp name starts with '.',
/// so the folder list hides it.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let dir = path.parent().unwrap_or(Path::new("."));
    let tmp = dir.join(format!(".{name}.{}.tmp", std::process::id()));
    let result = write_then_rename(&tmp, path, bytes);
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

fn write_then_rename(tmp: &Path, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = File::create(tmp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(tmp, path)
}

/// Text buffer for one note. Cursor column is a char index; display width is computed in ui.
pub struct Editor {
    pub path: PathBuf,
    pub lines: Vec<String>,
    pub row: usize,
    pub col: usize,
    pub scroll_y: usize,
    pub scroll_x: usize,
    pub dirty: bool,
    pub last_edit: Instant,
    /// Original line ending ("\r\n" or "\n"), restored on save.
    eol: &'static str,
    /// File modification time as last read or written by this editor.
    disk_mtime: Option<SystemTime>,
    /// Set when a save found the file changed outside. Autosave stops until a forced save.
    pub conflict: bool,
}

impl Editor {
    pub fn open(path: PathBuf) -> std::io::Result<Self> {
        let text = fs::read_to_string(&path)?;
        let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
        let lines = text.split(eol).map(str::to_string).collect();
        let disk_mtime = modified_time(&path);
        Ok(Editor {
            path,
            lines,
            row: 0,
            col: 0,
            scroll_y: 0,
            scroll_x: 0,
            dirty: false,
            last_edit: Instant::now(),
            eol,
            disk_mtime,
            conflict: false,
        })
    }

    /// True if the file's modification time differs from the one we last saw.
    /// If the file cannot be read right now (folder offline, deleted), this is false:
    /// the save itself reports that error and is retried.
    pub fn changed_on_disk(&self) -> bool {
        match fs::metadata(&self.path).and_then(|m| m.modified()) {
            Ok(now) => Some(now) != self.disk_mtime,
            Err(_) => false,
        }
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        if !self.dirty {
            return Ok(());
        }
        write_atomic(&self.path, self.lines.join(self.eol).as_bytes())?;
        self.disk_mtime = modified_time(&self.path);
        self.dirty = false;
        self.conflict = false;
        Ok(())
    }

    pub fn insert_char(&mut self, c: char) {
        let line = &mut self.lines[self.row];
        let at = byte_index(line, self.col);
        line.insert(at, c);
        self.col += 1;
        self.touch();
    }

    pub fn insert_newline(&mut self) {
        let line = &mut self.lines[self.row];
        let rest = line.split_off(byte_index(line, self.col));
        self.row += 1;
        self.lines.insert(self.row, rest);
        self.col = 0;
        self.touch();
    }

    pub fn backspace(&mut self) {
        if self.col > 0 {
            let line = &mut self.lines[self.row];
            let start = byte_index(line, self.col - 1);
            let end = byte_index(line, self.col);
            line.replace_range(start..end, "");
            self.col -= 1;
        } else if self.row > 0 {
            let line = self.lines.remove(self.row);
            self.row -= 1;
            self.col = self.lines[self.row].chars().count();
            self.lines[self.row].push_str(&line);
        } else {
            return;
        }
        self.touch();
    }

    pub fn delete(&mut self) {
        let len = self.lines[self.row].chars().count();
        if self.col < len {
            let line = &mut self.lines[self.row];
            let start = byte_index(line, self.col);
            let end = byte_index(line, self.col + 1);
            line.replace_range(start..end, "");
        } else if self.row + 1 < self.lines.len() {
            let next = self.lines.remove(self.row + 1);
            self.lines[self.row].push_str(&next);
        } else {
            return;
        }
        self.touch();
    }

    pub fn left(&mut self) {
        if self.col > 0 {
            self.col -= 1;
        } else if self.row > 0 {
            self.row -= 1;
            self.col = self.line_len();
        }
    }

    pub fn right(&mut self) {
        if self.col < self.line_len() {
            self.col += 1;
        } else if self.row + 1 < self.lines.len() {
            self.row += 1;
            self.col = 0;
        }
    }

    pub fn up(&mut self) {
        if self.row > 0 {
            self.row -= 1;
            self.col = self.col.min(self.line_len());
        }
    }

    pub fn down(&mut self) {
        if self.row + 1 < self.lines.len() {
            self.row += 1;
            self.col = self.col.min(self.line_len());
        }
    }

    pub fn home(&mut self) {
        self.col = 0;
    }

    pub fn end(&mut self) {
        self.col = self.line_len();
    }

    pub fn line_len(&self) -> usize {
        self.lines[self.row].chars().count()
    }

    fn touch(&mut self) {
        self.dirty = true;
        self.last_edit = Instant::now();
    }
}

fn byte_index(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buf(text: &str) -> Editor {
        Editor {
            path: PathBuf::new(),
            lines: text.split('\n').map(str::to_string).collect(),
            row: 0,
            col: 0,
            scroll_y: 0,
            scroll_x: 0,
            dirty: false,
            last_edit: Instant::now(),
            eol: "\n",
            disk_mtime: None,
            conflict: false,
        }
    }

    #[test]
    fn korean_insert_and_backspace() {
        let mut e = buf("");
        e.insert_char('가');
        e.insert_char('나');
        e.insert_char('다');
        e.col = 1;
        e.backspace();
        assert_eq!(e.lines[0], "나다");
        assert_eq!(e.col, 0);
    }

    #[test]
    fn newline_splits_and_backspace_joins() {
        let mut e = buf("abcd");
        e.col = 2;
        e.insert_newline();
        assert_eq!(e.lines, ["ab", "cd"]);
        e.backspace();
        assert_eq!(e.lines, ["abcd"]);
        assert_eq!(e.col, 2);
    }

    #[test]
    fn write_atomic_replaces_content_and_leaves_no_temp() {
        let dir = std::env::temp_dir().join(format!("ho-atomic-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("n.md");
        fs::write(&path, "old").unwrap();

        write_atomic(&path, "new 내용".as_bytes()).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "new 내용");
        let names: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["n.md"]);

        // Missing folder: the write fails and nothing is left behind.
        let gone = dir.join("missing").join("n.md");
        assert!(write_atomic(&gone, b"x").is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn delete_joins_next_line() {
        let mut e = buf("ab\ncd");
        e.col = 2;
        e.delete();
        assert_eq!(e.lines, ["abcd"]);
    }
}
