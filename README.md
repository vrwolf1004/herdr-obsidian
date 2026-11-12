# herdr-obsidian

[**English**](README.md) | [한국어](README.ko.md)

UI language: Korean

Browse and edit an [Obsidian](https://obsidian.md) vault from one herdr pane, without opening Obsidian. Notes are plain Markdown files, so any folder works, including one synced by Google Drive for desktop. Edits are saved automatically.

## Dashboard design

The editor is on the left. The vault list and the folder/note list are on the right. The status line at the bottom shows messages, prompts, and the `Ctrl+H` hint.

```text
┌ plan.md ● ──────────────────────────┐┌ 볼트 목록 (↑↓ Enter 전환) ─────────┐
│# Plan                               ││● C:\work\obsidian\herdr_test       │
│                                     ││  C:\work\obsidian\test             │
│- [ ] write the README               │└────────────────────────────────────┘
│- [x] autosave                       │┌ C:\work\obsidian\herdr_test ───────┐
│                                     ││▾ Projects                          │
│> notes are plain Markdown           ││    plan.md                         │
│                                     ││▸ Archive                           │
│                                     ││  welcome.md                        │
└─────────────────────────────────────┘└────────────────────────────────────┘
saved: C:\work\obsidian\herdr_test\Projects\plan.md             Ctrl+H 단축키
```

The vault list is the top-right panel. Select a vault with `↑`/`↓` and press `Enter` to switch; the folder/note list below then shows that vault. The current vault has a `●`. Headings, quotes, list items, and fenced code are colored; the text itself is shown as written.

This plugin runs as a full-screen terminal app in a pane, not as a block in the Agents sidebar.

Press `Ctrl+H` to show the shortcut list in a popup. Press `Esc` or `Ctrl+H` to close it.

## Installation

There is no `herdr-plugin.toml` yet, so `herdr plugin install` does not work. Build it and run the binary in a herdr pane:

```sh
git clone https://github.com/vrwolf1004/herdr-obsidian.git
cd herdr-obsidian
cargo build --release
target/release/herdr-obsidian "C:\work\obsidian\herdr_test"
```

A vault given on the command line is added to the vault list and opened. Without an argument, the first existing vault in the list is opened.

## Vault list

The list is kept in `config.toml`:

| OS | Path |
| --- | --- |
| Windows | `%APPDATA%\herdr-obsidian\config.toml` |
| Linux, macOS | `$XDG_CONFIG_HOME/herdr-obsidian/config.toml`, or `~/.config/herdr-obsidian/config.toml` |

```toml
vaults = ['C:\work\obsidian\herdr_test', 'C:\work\obsidian\test']
```

## Shortcuts

| Key | What it does |
| --- | --- |
| `Ctrl+O` | Open a folder: a path relative to the vault, or an absolute path (may be outside the vault) |
| `Ctrl+N` | Create a note in the list root (`.md` is added) |
| `Ctrl+G` | Create a folder inside the selected folder |
| `Ctrl+T` | Rename the selected note |
| `Ctrl+D` | Move the selected note to `.trash/` after confirming with `y` |
| `Ctrl+S` | Save now |
| `Ctrl+Q` | Save and quit |
| `Ctrl+H` | Show or hide the shortcut list |
| `Esc` | Switch focus (vaults → list → editor); close a prompt or popup |
| `↑` `↓` `Enter` `→` `←` | Move, open a note, expand or collapse a folder |
| `r` | Refresh the list |
| Mouse | Click to select or place the cursor, wheel to scroll |

## Saving

- A note is saved 0.8 seconds after the last keystroke.
- Saving writes a temporary file and renames it, so a note is never left half-written.
- If a save fails, it is retried every 3 seconds. Press `Ctrl+Q` twice to quit without saving.
- If the file changed on disk after it was opened (Obsidian, a sync client), autosave stops and a warning is shown. `Ctrl+S` overwrites the file with your text.
- Deleted notes go to `.trash/` under the list root. There is no restore screen; move the file back yourself.

## Release notes

### 0.5.0

- Vault list (top right): vaults come from `config.toml`; switch with `↑`/`↓` and `Enter`.
- Folder open (`Ctrl+O`): relative paths are inside the vault; absolute paths may point outside it.
- File operations: create note (`Ctrl+N`), create folder (`Ctrl+G`), rename (`Ctrl+T`), move to `.trash/` (`Ctrl+D`).
- Shortcut popup (`Ctrl+H`) and a `Ctrl+H` hint pinned to the status line.
- Safer saving: temporary file then rename; retry every 3 seconds after a failure; `Ctrl+Q` twice quits without saving.
- External change detection: autosave stops when the file changed on disk; `Ctrl+S` overwrites.
- Left arrow collapses an open folder, or moves to the parent folder.

## Requirements

- Windows, Linux, or macOS (developed and tested on Windows)
- herdr 0.9.0 or later
- Rust (stable), to build

## Documentation

- [Usage guide (English)](docs/usage.md)
- [사용법 (한국어)](docs/usage.ko.md)
- [TODO](docs/todo.md)

## Development

```sh
cargo test
cargo clippy --all-targets
cargo fmt --check
```

## License

MIT

## Support the project

If this project is useful to you, consider buying the developer a coffee.

[![Support on Ko-fi](https://img.shields.io/badge/Ko--fi-Support-FF5E5B?style=for-the-badge&logo=buymeacoffee&logoColor=white)](https://ko-fi.com/dev_ohyt)
