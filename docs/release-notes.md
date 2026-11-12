# Release notes

[한국어](release-notes.ko.md)

## 0.5.0

- Vault list (top right): vaults come from `config.toml`; switch with `↑`/`↓` and `Enter`.
- Folder open (`Ctrl+O`): relative paths are inside the vault; absolute paths may point outside it.
- File operations: create note (`Ctrl+N`), create folder (`Ctrl+G`), rename (`Ctrl+T`), move to `.trash/` (`Ctrl+D`).
- Shortcut popup (`Ctrl+H`) and a `Ctrl+H` hint pinned to the status line.
- Safer saving: temporary file then rename; retry every 3 seconds after a failure; `Ctrl+Q` twice quits without saving.
- External change detection: autosave stops when the file changed on disk; `Ctrl+S` overwrites.
- Left arrow collapses an open folder, or moves to the parent folder.
- UI in 7 languages (en, ko, ru, fr, es, de, ja); ru, fr, es, de, ja are unreviewed drafts.
