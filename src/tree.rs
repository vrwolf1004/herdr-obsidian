use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct Row {
    pub path: PathBuf,
    pub is_dir: bool,
    pub depth: usize,
    pub name: String,
}

/// Flattened view of the vault. Only expanded folders are read from disk.
pub struct Tree {
    pub root: PathBuf,
    pub expanded: HashSet<PathBuf>,
    pub rows: Vec<Row>,
}

impl Tree {
    pub fn new(root: PathBuf) -> Self {
        let mut tree = Tree {
            root,
            expanded: HashSet::new(),
            rows: Vec::new(),
        };
        tree.refresh();
        tree
    }

    pub fn refresh(&mut self) {
        let mut rows = Vec::new();
        collect(&self.root, 0, &self.expanded, &mut rows);
        self.rows = rows;
    }

    /// Show `root` as the top of the list. Expansion state belongs to the old root, so it is reset.
    pub fn set_root(&mut self, root: PathBuf) {
        self.root = root;
        self.expanded.clear();
        self.refresh();
    }

    /// Index of the parent row: the nearest row above with a smaller depth. None at top level.
    pub fn parent_index(&self, index: usize) -> Option<usize> {
        let depth = self.rows.get(index)?.depth;
        (0..index).rev().find(|&i| self.rows[i].depth < depth)
    }

    /// Toggle a folder open/closed. Returns false if the row is a file.
    pub fn toggle(&mut self, index: usize) -> bool {
        let Some(row) = self.rows.get(index) else {
            return false;
        };
        if !row.is_dir {
            return false;
        }
        let path = row.path.clone();
        if !self.expanded.remove(&path) {
            self.expanded.insert(path);
        }
        self.refresh();
        true
    }
}

/// Markdown files and folders only; hidden entries (.obsidian, .trash, ...) are skipped.
fn collect(dir: &Path, depth: usize, expanded: &HashSet<PathBuf>, out: &mut Vec<Row>) {
    let Ok(read) = fs::read_dir(dir) else { return };
    let mut entries: Vec<(PathBuf, String, bool)> = read
        .filter_map(Result::ok)
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                return None;
            }
            let path = e.path();
            let is_dir = path.is_dir();
            if !is_dir && !name.to_lowercase().ends_with(".md") {
                return None;
            }
            Some((path, name, is_dir))
        })
        .collect();
    // Folders first, then names case-insensitively.
    entries.sort_by(|a, b| {
        b.2.cmp(&a.2)
            .then_with(|| a.1.to_lowercase().cmp(&b.1.to_lowercase()))
    });
    for (path, name, is_dir) in entries {
        let open = is_dir && expanded.contains(&path);
        out.push(Row {
            path: path.clone(),
            is_dir,
            depth,
            name,
        });
        if open {
            collect(&path, depth + 1, expanded, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_index_walks_up_levels() {
        let dir = std::env::temp_dir().join(format!("ho-parent-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("A").join("B")).unwrap();
        fs::write(dir.join("A").join("B").join("n.md"), "").unwrap();
        let mut tree = Tree::new(dir.clone());
        tree.expanded.insert(dir.join("A"));
        tree.expanded.insert(dir.join("A").join("B"));
        tree.refresh();
        // rows: A(0), B(1), n.md(2)
        assert_eq!(tree.parent_index(2), Some(1));
        assert_eq!(tree.parent_index(1), Some(0));
        assert_eq!(tree.parent_index(0), None);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn lists_folders_first_and_skips_hidden() {
        let dir = std::env::temp_dir().join(format!("ho-tree-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("Projects")).unwrap();
        fs::create_dir_all(dir.join(".obsidian")).unwrap();
        fs::write(dir.join("b.md"), "").unwrap();
        fs::write(dir.join("A.md"), "").unwrap();
        fs::write(dir.join("img.png"), "").unwrap();

        let mut tree = Tree::new(dir.clone());
        let names: Vec<&str> = tree.rows.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["Projects", "A.md", "b.md"]);

        fs::write(dir.join("Projects").join("note.md"), "").unwrap();
        assert!(tree.toggle(0));
        assert_eq!(tree.rows[1].name, "note.md");
        assert_eq!(tree.rows[1].depth, 1);
        assert!(!tree.toggle(1));

        let _ = fs::remove_dir_all(&dir);
    }
}
