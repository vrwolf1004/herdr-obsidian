mod config;
mod editor;
mod files;
mod i18n;
mod tree;
mod ui;
mod vaults;

use std::env;
use std::io::{self, stdout};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant, SystemTime};

use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
    KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::crossterm::execute;
use ratatui::layout::{Position, Rect};
use ratatui::DefaultTerminal;
use unicode_width::UnicodeWidthChar;

use editor::Editor;
use i18n::t;
use tree::Tree;

/// Save this long after the last keystroke.
const DEBOUNCE: Duration = Duration::from_millis(800);
const IDLE_POLL: Duration = Duration::from_millis(1000);
/// Wait this long before retrying a failed autosave (no tight loop on a locked file).
const RETRY_INTERVAL: Duration = Duration::from_secs(3);

#[derive(Clone, Copy, PartialEq)]
pub enum Focus {
    /// Vault list (top right).
    Vaults,
    /// Folder/note list of the current vault (bottom right).
    Tree,
    Editor,
}

#[derive(Clone)]
pub enum PromptKind {
    /// Ctrl+O: folder path (relative to vault, or absolute).
    Folder,
    /// Ctrl+N: name of a new note in the current list root.
    NewNote,
    /// Ctrl+T: new name for this file; input starts with the current name.
    Rename(PathBuf),
    /// Ctrl+G: name of a new folder inside this parent folder.
    NewFolder(PathBuf),
    /// Ctrl+E: absolute path of a folder to add to the vault list.
    AddVault,
}

pub struct Prompt {
    pub kind: PromptKind,
    pub input: String,
}

pub struct App {
    /// Current vault root (`vaults[current_vault]`); relative folder input is resolved against it.
    pub vault: PathBuf,
    /// All vaults from the config file, in list order.
    pub vaults: Vec<PathBuf>,
    pub current_vault: usize,
    pub vault_selected: usize,
    pub vault_offset: usize,
    pub vault_area: Rect,
    pub tree: Tree,
    pub selected: usize,
    pub tree_offset: usize,
    pub editor: Option<Editor>,
    pub focus: Focus,
    pub status: String,
    /// Single-line input in the status bar. `Some` while open.
    pub prompt: Option<Prompt>,
    /// File waiting for delete confirmation (Ctrl+D). `Some` while waiting.
    pub confirm_delete: Option<PathBuf>,
    /// Vault index waiting for removal confirmation (Delete in the vault list). `Some` while waiting.
    pub confirm_vault: Option<usize>,
    /// Content areas (inside borders) from the last draw, used for mouse hit-testing.
    pub tree_area: Rect,
    pub editor_area: Rect,
    /// Ctrl+P: show the open note as rendered markdown (read-only).
    pub preview: bool,
    pub quit: bool,
    /// When a failed save may be retried. `None` when the last save succeeded.
    pub retry_at: Option<Instant>,
    /// Ctrl+Q was pressed while saving failed; a second Ctrl+Q quits without saving.
    pub quit_armed: bool,
    /// Shortcut list popup (Ctrl+H). While open it takes all keys except Esc / Ctrl+H.
    pub help_open: bool,
}

impl App {
    fn new(vaults: Vec<PathBuf>, current: usize) -> Self {
        let vault = vaults[current].clone();
        App {
            tree: Tree::new(vault.clone()),
            vault,
            vaults,
            current_vault: current,
            vault_selected: current,
            vault_offset: 0,
            vault_area: Rect::default(),
            selected: 0,
            tree_offset: 0,
            editor: None,
            focus: Focus::Tree,
            status: t("status.default", &[]),
            prompt: None,
            confirm_delete: None,
            confirm_vault: None,
            tree_area: Rect::default(),
            editor_area: Rect::default(),
            preview: false,
            quit: false,
            retry_at: None,
            quit_armed: false,
            help_open: false,
        }
    }

    fn open_folder(&mut self, input: &str) {
        match resolve_folder(&self.vault, input) {
            Ok(dir) => {
                self.status = t("msg.folder_opened", &[("path", &dir.display().to_string())]);
                self.tree.set_root(dir);
                self.selected = 0;
                self.tree_offset = 0;
            }
            Err(msg) => self.status = msg,
        }
    }

    fn prompt_key(&mut self, key: KeyEvent) {
        let Some(prompt) = self.prompt.as_mut() else {
            return;
        };
        match key.code {
            KeyCode::Char(c)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    || key.modifiers.contains(KeyModifiers::ALT) =>
            {
                prompt.input.push(c)
            }
            KeyCode::Backspace => {
                prompt.input.pop();
            }
            KeyCode::Esc => self.prompt = None,
            KeyCode::Enter => {
                if let Some(prompt) = self.prompt.take() {
                    self.submit(prompt);
                }
            }
            _ => {}
        }
    }

    fn submit(&mut self, prompt: Prompt) {
        let input = prompt.input.trim().to_string();
        match prompt.kind {
            PromptKind::Folder => self.open_folder(&input),
            PromptKind::NewNote => self.new_note(&input),
            PromptKind::Rename(from) => self.rename_selected(&from, &input),
            PromptKind::NewFolder(parent) => self.new_folder(&parent, &input),
            PromptKind::AddVault => self.add_vault(&input),
        }
    }

    /// Add a folder to the vault list (or switch to it if already listed), then save config.toml.
    fn add_vault(&mut self, input: &str) {
        // Explorer's "Copy as path" wraps the path in quotes.
        let input = input.trim_matches('"');
        if input.is_empty() {
            self.status = t("err.not_folder", &[("input", input)]);
            return;
        }
        let dir = match vaults::absolute(input) {
            Ok(dir) => dir,
            Err(msg) => {
                self.status = msg;
                return;
            }
        };
        if let Some(index) = vaults::position(&self.vaults, &dir) {
            self.switch_vault(index);
            return;
        }
        let mut list = self.vaults.clone();
        list.push(dir);
        if let Err(e) = self.save_vaults(&list) {
            self.status = e;
            return;
        }
        self.vaults = list;
        self.switch_vault(self.vaults.len() - 1);
    }

    /// Write `list` as the vault list in config.toml. Other settings are kept.
    fn save_vaults(&self, list: &[PathBuf]) -> Result<(), String> {
        let config_file = config::config_path().ok_or_else(|| t("err.no_config_dir", &[]))?;
        let mut cfg = config::load(&config_file)?;
        cfg.vaults = list.to_vec();
        config::save(&config_file, &cfg)
    }

    fn start_new_folder(&mut self) {
        // Selected folder, or the folder holding the selected file; vault root if nothing is selected.
        let parent = match self.tree.rows.get(self.selected) {
            Some(row) if row.is_dir => row.path.clone(),
            Some(row) => row
                .path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| self.tree.root.clone()),
            None => self.tree.root.clone(),
        };
        self.prompt = Some(Prompt {
            kind: PromptKind::NewFolder(parent),
            input: String::new(),
        });
    }

    fn new_folder(&mut self, parent: &Path, input: &str) {
        let created = files::check_name(input).and_then(|name| files::create_folder(parent, &name));
        match created {
            Ok(path) => {
                // Show the new folder: expand its parent first.
                self.tree.expanded.insert(parent.to_path_buf());
                self.tree.refresh();
                self.select_path(&path);
                self.status = t(
                    "msg.folder_created",
                    &[("path", &path.display().to_string())],
                );
            }
            Err(msg) => self.status = msg,
        }
    }

    /// ← : collapse an open folder; otherwise move to the parent folder.
    fn go_left(&mut self) {
        let Some(row) = self.tree.rows.get(self.selected) else {
            return;
        };
        if row.is_dir && self.tree.expanded.contains(&row.path) {
            self.tree.toggle(self.selected);
            self.clamp_selection();
        } else if let Some(parent) = self.tree.parent_index(self.selected) {
            self.selected = parent;
        }
    }

    /// Move the tree selection onto `path` if it is visible.
    fn select_path(&mut self, path: &Path) {
        if let Some(index) = self.tree.rows.iter().position(|r| r.path == path) {
            self.selected = index;
        }
    }

    fn new_note(&mut self, input: &str) {
        let created =
            files::note_name(input).and_then(|name| files::create_note(&self.tree.root, &name));
        match created {
            Ok(path) => {
                self.tree.refresh();
                self.select_path(&path);
                // Open it through the normal path so the current note is saved first.
                self.activate(self.selected);
                self.status = t("msg.note_created", &[("path", &path.display().to_string())]);
            }
            Err(msg) => self.status = msg,
        }
    }

    fn selected_file(&mut self) -> Option<(PathBuf, String)> {
        let row = self.tree.rows.get(self.selected)?;
        if row.is_dir {
            self.status = t("msg.folder_not_supported", &[]);
            return None;
        }
        Some((row.path.clone(), row.name.clone()))
    }

    fn start_rename(&mut self) {
        if let Some((path, name)) = self.selected_file() {
            self.prompt = Some(Prompt {
                kind: PromptKind::Rename(path),
                input: name,
            });
        }
    }

    fn rename_selected(&mut self, from: &Path, input: &str) {
        let name = match files::note_name(input) {
            Ok(name) => name,
            Err(msg) => {
                self.status = msg;
                return;
            }
        };
        // Save before moving: a later autosave must not write to the old path.
        if !self.save_now() {
            return;
        }
        match files::rename_note(from, &name) {
            Ok(to) => {
                if let Some(ed) = self.editor.as_mut().filter(|e| e.path == from) {
                    ed.path = to.clone();
                }
                self.tree.refresh();
                self.select_path(&to);
                self.status = t("msg.renamed", &[("path", &to.display().to_string())]);
            }
            Err(msg) => self.status = msg,
        }
    }

    fn start_delete(&mut self) {
        if let Some((path, name)) = self.selected_file() {
            let unsaved = self
                .editor
                .as_ref()
                .is_some_and(|e| e.path == path && e.dirty);
            let warn = if unsaved {
                t("msg.unsaved_suffix", &[])
            } else {
                String::new()
            };
            self.status = t("msg.confirm_trash", &[("name", &name), ("warn", &warn)]);
            self.confirm_delete = Some(path);
        }
    }

    fn confirm_key(&mut self, key: KeyEvent) {
        let Some(path) = self.confirm_delete.take() else {
            return;
        };
        if key.code != KeyCode::Char('y') && key.code != KeyCode::Char('Y') {
            self.status = t("msg.trash_cancelled", &[]);
            return;
        }
        match files::trash_note(&self.tree.root, &path, SystemTime::now()) {
            Ok(target) => {
                // Drop the editor for the moved file; otherwise autosave would recreate it.
                if self.editor.as_ref().is_some_and(|e| e.path == path) {
                    self.editor = None;
                    self.focus = Focus::Tree;
                }
                self.tree.refresh();
                self.clamp_selection();
                self.status = t("msg.trashed", &[("path", &target.display().to_string())]);
            }
            Err(msg) => self.status = msg,
        }
    }

    /// Write the open note if it has unsaved edits. Returns false on failure or conflict.
    pub fn save_now(&mut self) -> bool {
        self.save_with(false)
    }

    /// `force` skips the external-change check (Ctrl+S while a conflict is flagged).
    fn save_with(&mut self, force: bool) -> bool {
        let Some(ed) = self.editor.as_mut() else {
            return true;
        };
        if !ed.dirty {
            return true;
        }
        if !force && ed.changed_on_disk() {
            // Keep the edits in memory; nothing is written until the user forces it.
            ed.conflict = true;
            self.status = t("msg.conflict", &[]);
            return false;
        }
        match ed.save() {
            Ok(()) => {
                self.retry_at = None;
                self.status = t("msg.saved", &[("path", &ed.path.display().to_string())]);
                true
            }
            Err(e) => {
                self.retry_at = Some(Instant::now() + RETRY_INTERVAL);
                self.status = t(
                    "msg.save_failed",
                    &[
                        ("secs", &RETRY_INTERVAL.as_secs().to_string()),
                        ("error", &e.to_string()),
                    ],
                );
                false
            }
        }
    }

    /// When the next autosave should run: debounce after the last keystroke,
    /// and never before the retry time after a failed save. `None` if nothing is dirty.
    fn autosave_at(&self) -> Option<Instant> {
        let ed = self.editor.as_ref().filter(|e| e.dirty && !e.conflict)?;
        let debounced = ed.last_edit + DEBOUNCE;
        Some(self.retry_at.map_or(debounced, |r| r.max(debounced)))
    }

    fn clamp_selection(&mut self) {
        self.selected = self.selected.min(self.tree.rows.len().saturating_sub(1));
    }

    fn activate(&mut self, index: usize) {
        let Some(row) = self.tree.rows.get(index).cloned() else {
            return;
        };
        self.selected = index;
        if row.is_dir {
            self.tree.toggle(index);
            self.clamp_selection();
            return;
        }
        if self.editor.as_ref().is_some_and(|e| e.path == row.path) {
            self.focus = Focus::Editor;
            return;
        }
        // Never drop unsaved edits of the previous note.
        if !self.save_now() {
            return;
        }
        match Editor::open(row.path) {
            Ok(ed) => {
                self.status = t("msg.opened", &[("name", &row.name)]);
                self.editor = Some(ed);
                self.focus = Focus::Editor;
            }
            Err(e) => self.status = t("msg.open_failed", &[("error", &e.to_string())]),
        }
    }

    fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Vaults => Focus::Tree,
            Focus::Tree if self.editor.is_some() => Focus::Editor,
            _ => Focus::Tree,
        };
    }

    /// Make `vaults[index]` the current vault. Saves the open note first.
    fn switch_vault(&mut self, index: usize) {
        let Some(path) = self.vaults.get(index).cloned() else {
            return;
        };
        if index == self.current_vault {
            self.focus = Focus::Tree;
            return;
        }
        if !path.is_dir() {
            self.status = t(
                "msg.vault_missing",
                &[("path", &path.display().to_string())],
            );
            return;
        }
        if !self.save_now() {
            return;
        }
        self.editor = None;
        self.current_vault = index;
        self.vault_selected = index;
        self.vault = path.clone();
        self.tree.set_root(path.clone());
        self.selected = 0;
        self.tree_offset = 0;
        self.focus = Focus::Tree;
        self.status = t(
            "msg.vault_switched",
            &[("path", &path.display().to_string())],
        );
    }

    fn vault_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => self.vault_selected = self.vault_selected.saturating_sub(1),
            KeyCode::Down => {
                if self.vault_selected + 1 < self.vaults.len() {
                    self.vault_selected += 1;
                } else {
                    self.focus = Focus::Tree;
                }
            }
            KeyCode::Enter => self.switch_vault(self.vault_selected),
            KeyCode::Delete => self.start_remove_vault(),
            _ => {}
        }
    }

    /// Ask before taking the selected vault off the list. The folder itself is never touched.
    fn start_remove_vault(&mut self) {
        let index = self.vault_selected;
        let Some(path) = self.vaults.get(index) else {
            return;
        };
        if self.vaults.len() <= 1 {
            self.status = t("msg.vault_last", &[]);
            return;
        }
        self.status = t(
            "msg.confirm_vault_remove",
            &[("name", &path.display().to_string())],
        );
        self.confirm_vault = Some(index);
    }

    fn confirm_vault_key(&mut self, key: KeyEvent) {
        let Some(index) = self.confirm_vault.take() else {
            return;
        };
        if key.code != KeyCode::Char('y') && key.code != KeyCode::Char('Y') {
            self.status = t("msg.vault_remove_cancelled", &[]);
            return;
        }
        self.remove_vault(index);
    }

    /// Drop `vaults[index]` from the list and from config.toml. Switches vault if it was the current one.
    fn remove_vault(&mut self, index: usize) {
        if index == self.current_vault && !self.save_now() {
            return;
        }
        let removed = self.vaults[index].clone();
        let mut list = self.vaults.clone();
        list.remove(index);
        if let Err(e) = self.save_vaults(&list) {
            self.status = e;
            return;
        }

        if index == self.current_vault {
            let next = index.min(list.len() - 1);
            self.editor = None;
            self.current_vault = next;
            self.vault = list[next].clone();
            self.tree.set_root(self.vault.clone());
            self.selected = 0;
            self.tree_offset = 0;
            self.focus = Focus::Tree;
        } else if index < self.current_vault {
            self.current_vault -= 1;
        }
        self.vaults = list;
        self.vault_selected = index.min(self.vaults.len() - 1);
        self.vault_offset = 0;
        self.status = t(
            "msg.vault_removed",
            &[("path", &removed.display().to_string())],
        );
    }

    fn tree_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                if self.selected == 0 {
                    self.focus = Focus::Vaults;
                } else {
                    self.selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected + 1 < self.tree.rows.len() {
                    self.selected += 1;
                }
            }
            KeyCode::Enter | KeyCode::Right => self.activate(self.selected),
            KeyCode::Left => self.go_left(),
            KeyCode::Char('r') => {
                self.tree.refresh();
                self.clamp_selection();
            }
            KeyCode::Tab if self.editor.is_some() => self.focus = Focus::Editor,
            _ => {}
        }
    }

    fn editor_key(&mut self, key: KeyEvent) {
        // Preview is read-only: only cursor movement goes through.
        let moves_only = self.preview;
        let Some(ed) = self.editor.as_mut() else {
            return;
        };
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        if moves_only
            && !matches!(
                key.code,
                KeyCode::Left
                    | KeyCode::Right
                    | KeyCode::Up
                    | KeyCode::Down
                    | KeyCode::Home
                    | KeyCode::End
                    | KeyCode::PageUp
                    | KeyCode::PageDown
            )
        {
            return;
        }
        match key.code {
            // Ctrl+letter is a shortcut; AltGr (Ctrl+Alt) still types.
            KeyCode::Char(c) if !ctrl || alt => ed.insert_char(c),
            KeyCode::Enter => ed.insert_newline(),
            KeyCode::Tab => (0..4).for_each(|_| ed.insert_char(' ')),
            KeyCode::Backspace => ed.backspace(),
            KeyCode::Delete => ed.delete(),
            KeyCode::Left => ed.left(),
            KeyCode::Right => ed.right(),
            KeyCode::Up => ed.up(),
            KeyCode::Down => ed.down(),
            KeyCode::Home => ed.home(),
            KeyCode::End => ed.end(),
            KeyCode::PageUp => (0..10).for_each(|_| ed.up()),
            KeyCode::PageDown => (0..10).for_each(|_| ed.down()),
            _ => {}
        }
    }

    fn click(&mut self, x: u16, y: u16) {
        let pos = Position::new(x, y);
        if self.vault_area.contains(pos) {
            self.focus = Focus::Vaults;
            let index = self.vault_offset + (y - self.vault_area.y) as usize;
            if index < self.vaults.len() {
                self.vault_selected = index;
                self.switch_vault(index);
            }
        } else if self.tree_area.contains(pos) {
            self.focus = Focus::Tree;
            let index = self.tree_offset + (y - self.tree_area.y) as usize;
            if index < self.tree.rows.len() {
                self.activate(index);
            }
        } else if self.editor_area.contains(pos) {
            self.focus = Focus::Editor;
            if let Some(ed) = self.editor.as_mut() {
                let row = ed.scroll_y + (y - self.editor_area.y) as usize;
                let width = ed.scroll_x + (x - self.editor_area.x) as usize;
                ed.row = row.min(ed.lines.len() - 1);
                ed.col = col_at(&ed.lines[ed.row], width);
            }
        }
    }
}

/// Char index whose display cell range covers `width` cells from the start of `line`.
fn col_at(line: &str, width: usize) -> usize {
    let mut used = 0;
    for (i, c) in line.chars().enumerate() {
        used += c.width().unwrap_or(0);
        if used > width {
            return i;
        }
    }
    line.chars().count()
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if matches!(args.first().map(String::as_str), Some("-h" | "--help")) {
        println!("{}", t("usage", &[]));
        return ExitCode::SUCCESS;
    }
    let (vaults, current) = match vaults::startup(args.first()) {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("herdr-obsidian: {msg}");
            return ExitCode::from(2);
        }
    };
    match run(vaults, current) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("herdr-obsidian: {e}");
            ExitCode::from(1)
        }
    }
}

/// Map user input to a folder. Empty input means the vault root.
/// Relative input is taken from the vault root and must stay inside the vault.
/// Absolute input may be any existing folder, inside or outside the vault.
fn resolve_folder(vault: &Path, input: &str) -> Result<PathBuf, String> {
    let input = input.trim();
    if input.is_empty() {
        return Ok(vault.to_path_buf());
    }
    if Path::new(input).is_absolute() {
        // Absolute paths may point outside the vault; only check that the folder exists.
        let dir = PathBuf::from(input);
        return if dir.is_dir() {
            Ok(dir)
        } else {
            Err(t("err.folder_not_found", &[("input", input)]))
        };
    }
    let candidate = vault.join(input);
    let real_vault = vault
        .canonicalize()
        .map_err(|e| t("err.vault_io", &[("error", &e.to_string())]))?;
    let real = candidate
        .canonicalize()
        .map_err(|_| t("err.folder_not_found", &[("input", input)]))?;
    if !real.is_dir() {
        return Err(t("err.not_folder", &[("input", input)]));
    }
    match real.strip_prefix(&real_vault) {
        Ok(rel) if rel.as_os_str().is_empty() => Ok(vault.to_path_buf()),
        Ok(rel) => Ok(vault.join(rel)),
        Err(_) => Err(t("err.outside_vault", &[("input", input)])),
    }
}

fn run(vaults: Vec<PathBuf>, current: usize) -> io::Result<()> {
    let mut app = App::new(vaults, current);
    let mut terminal = ratatui::init();
    let result =
        execute!(stdout(), EnableMouseCapture).and_then(|()| event_loop(&mut terminal, &mut app));
    let _ = execute!(stdout(), DisableMouseCapture);
    ratatui::restore();
    result
}

fn event_loop(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;
        let timeout = match app.autosave_at() {
            Some(at) => at.saturating_duration_since(Instant::now()),
            None => IDLE_POLL,
        };
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => handle_key(app, key),
                Event::Mouse(m) => handle_mouse(app, m),
                _ => {}
            }
        }
        if app.autosave_at().is_some_and(|at| Instant::now() >= at) {
            app.save_now();
        }
        if app.quit {
            return Ok(());
        }
    }
}

/// Ctrl+H. Some terminals send it as Backspace, so Backspace with Ctrl counts too.
/// Plain Backspace (no modifier) is never a help key, so text editing is unaffected.
fn is_help_key(key: &KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL)
        && !key.modifiers.contains(KeyModifiers::ALT)
        && matches!(key.code, KeyCode::Char('h') | KeyCode::Backspace)
}

fn handle_key(app: &mut App, key: KeyEvent) {
    let is_quit = key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q');
    if !is_quit {
        app.quit_armed = false;
    }
    if app.confirm_vault.is_some() {
        app.confirm_vault_key(key);
        return;
    }
    if app.confirm_delete.is_some() {
        app.confirm_key(key);
        return;
    }
    if app.help_open {
        if is_help_key(&key) || key.code == KeyCode::Esc {
            app.help_open = false;
        }
        return;
    }
    // Inside a prompt, Backspace keeps deleting text, so help does not open there.
    if is_help_key(&key) && app.prompt.is_none() {
        app.help_open = true;
        return;
    }
    if app.prompt.is_some() {
        app.prompt_key(key);
        return;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('q') => {
                // Saved (or nothing to save), or the user already pressed Ctrl+Q once after a failed save.
                if app.save_now() || app.quit_armed {
                    app.quit = true;
                } else {
                    app.quit_armed = true;
                    app.status = t("msg.quit_armed", &[]);
                }
                return;
            }
            KeyCode::Char('s') => {
                // After a conflict warning, the second Ctrl+S overwrites the external change.
                let force = app.editor.as_ref().is_some_and(|e| e.conflict);
                app.save_with(force);
                return;
            }
            KeyCode::Char('o') => {
                app.prompt = Some(Prompt {
                    kind: PromptKind::Folder,
                    input: String::new(),
                });
                return;
            }
            KeyCode::Char('n') => {
                app.prompt = Some(Prompt {
                    kind: PromptKind::NewNote,
                    input: String::new(),
                });
                return;
            }
            KeyCode::Char('t') => {
                app.start_rename();
                return;
            }
            KeyCode::Char('d') => {
                app.start_delete();
                return;
            }
            KeyCode::Char('g') => {
                app.start_new_folder();
                return;
            }
            KeyCode::Char('p') => {
                if app.editor.is_some() {
                    app.preview = !app.preview;
                }
                return;
            }
            KeyCode::Char('e') => {
                app.prompt = Some(Prompt {
                    kind: PromptKind::AddVault,
                    input: String::new(),
                });
                return;
            }
            _ => {}
        }
    }
    if key.code == KeyCode::Esc {
        app.toggle_focus();
        return;
    }
    match app.focus {
        Focus::Vaults => app.vault_key(key),
        Focus::Tree => app.tree_key(key),
        Focus::Editor => app.editor_key(key),
    }
}

fn handle_mouse(app: &mut App, m: MouseEvent) {
    if app.help_open {
        return;
    }
    match m.kind {
        MouseEventKind::Down(MouseButton::Left) => app.click(m.column, m.row),
        MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => {
            let delta: isize = if m.kind == MouseEventKind::ScrollDown {
                3
            } else {
                -3
            };
            let over_editor = app.editor_area.contains(Position::new(m.column, m.row));
            if let Some(ed) = app.editor.as_mut().filter(|_| over_editor) {
                let max = ed.lines.len().saturating_sub(1) as isize;
                ed.scroll_y = (ed.scroll_y as isize + delta).clamp(0, max) as usize;
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{col_at, resolve_folder, App};
    use crate::editor::Editor;
    use crate::i18n::t;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::fs;

    /// App with a dirty note whose folder was deleted, so every save fails.
    fn app_with_failing_save(tag: &str) -> App {
        let dir = std::env::temp_dir().join(format!("ho-fail-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("n.md");
        fs::write(&path, "a").unwrap();
        let mut app = App::new(vec![dir.clone()], 0);
        let mut ed = Editor::open(path).unwrap();
        ed.insert_char('b');
        app.editor = Some(ed);
        fs::remove_dir_all(&dir).unwrap();
        app
    }

    #[test]
    fn failed_save_waits_before_retry() {
        let mut app = app_with_failing_save("retry");
        assert!(!app.save_now());
        let retry = app.retry_at.expect("retry time set after failure");
        // Autosave is not due until the retry time, so no tight loop.
        assert!(app.autosave_at().unwrap() >= retry);
        assert!(app.autosave_at().unwrap() > std::time::Instant::now());
    }

    #[test]
    fn second_ctrl_q_quits_after_failed_save() {
        let mut app = app_with_failing_save("quit");
        let ctrl_q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        super::handle_key(&mut app, ctrl_q);
        assert!(!app.quit && app.quit_armed);
        super::handle_key(&mut app, ctrl_q);
        assert!(app.quit);
    }

    #[test]
    fn other_key_disarms_quit() {
        let mut app = app_with_failing_save("disarm");
        let ctrl_q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        super::handle_key(&mut app, ctrl_q);
        super::handle_key(&mut app, KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert!(!app.quit_armed);
        super::handle_key(&mut app, ctrl_q);
        assert!(!app.quit);
    }

    #[test]
    fn resolve_folder_stays_inside_vault() {
        let vault = std::env::temp_dir().join(format!("ho-folder-{}", std::process::id()));
        let _ = fs::remove_dir_all(&vault);
        fs::create_dir_all(vault.join("Projects").join("Sub")).unwrap();

        assert_eq!(resolve_folder(&vault, "  ").unwrap(), vault);
        assert_eq!(
            resolve_folder(&vault, "Projects").unwrap(),
            vault.join("Projects")
        );
        assert_eq!(
            resolve_folder(&vault, "Projects/Sub").unwrap(),
            vault.join("Projects").join("Sub")
        );
        assert_eq!(resolve_folder(&vault, "Projects/..").unwrap(), vault);
        assert_eq!(
            resolve_folder(&vault, "..").unwrap_err(),
            t("err.outside_vault", &[("input", "..")])
        );
        assert_eq!(
            resolve_folder(&vault, "missing").unwrap_err(),
            t("err.folder_not_found", &[("input", "missing")])
        );
        let _ = fs::remove_dir_all(&vault);
    }

    #[test]
    fn resolve_folder_allows_absolute_path_outside_vault() {
        let base = std::env::temp_dir().join(format!("ho-abs-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let vault = base.join("vault");
        let outside = base.join("elsewhere");
        fs::create_dir_all(&vault).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("note.md"), "").unwrap();

        let input = outside.to_string_lossy().into_owned();
        assert_eq!(resolve_folder(&vault, &input).unwrap(), outside);
        assert!(resolve_folder(&vault, &base.join("missing").to_string_lossy()).is_err());
        // A file is not a folder.
        assert!(resolve_folder(&vault, &outside.join("note.md").to_string_lossy()).is_err());

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn external_change_blocks_save_until_forced() {
        let dir = std::env::temp_dir().join(format!("ho-conflict-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("n.md");
        fs::write(&path, "a").unwrap();
        let mut app = App::new(vec![dir.clone()], 0);
        let mut ed = Editor::open(path.clone()).unwrap();
        ed.insert_char('b');
        app.editor = Some(ed);

        // Another program writes the file after we opened it.
        std::thread::sleep(std::time::Duration::from_millis(50));
        fs::write(&path, "outside").unwrap();

        assert!(!app.save_with(false));
        assert!(app.editor.as_ref().unwrap().conflict);
        assert!(
            app.autosave_at().is_none(),
            "autosave must stop on conflict"
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), "outside");

        // Ctrl+S after the warning overwrites with the in-memory text.
        assert!(app.save_with(true));
        assert_eq!(fs::read_to_string(&path).unwrap(), "ba");
        assert!(!app.editor.as_ref().unwrap().conflict);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn col_at_counts_wide_chars_as_two_cells() {
        // "가나b": cells 0-1 = 가, 2-3 = 나, 4 = b
        assert_eq!(col_at("가나b", 0), 0);
        assert_eq!(col_at("가나b", 1), 0);
        assert_eq!(col_at("가나b", 2), 1);
        assert_eq!(col_at("가나b", 4), 2);
        assert_eq!(col_at("가나b", 99), 3);
    }
}
