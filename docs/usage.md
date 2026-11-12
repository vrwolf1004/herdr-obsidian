# Usage

[한국어](usage.ko.md)

## Vault list

Vaults come from `config.toml`. The file is created on first run.

| OS | Path |
| --- | --- |
| Windows | `%APPDATA%\herdr-obsidian\config.toml` |
| Linux, macOS | `$XDG_CONFIG_HOME/herdr-obsidian/config.toml`, or `~/.config/herdr-obsidian/config.toml` |

```toml
vaults = ['C:\work\obsidian\herdr_test', 'C:\work\obsidian\test']
```

- A vault given on the command line is added to the list (if new) and opened.
- Without an argument, the first existing vault in the list is opened.
- A vault folder that no longer exists is shown as `(없음)`.
- The list is only changed by adding a command-line vault. Remove an entry by editing the file.

## Shortcuts

Press `Ctrl+H` in the app to see this list in a popup.

| Key | What it does |
| --- | --- |
| `Ctrl+O` | Open a folder: a path relative to the vault, or an absolute path (may be outside the vault) |
| `Ctrl+N` | Create a note in the list root (`.md` is added) |
| `Ctrl+G` | Create a folder inside the selected folder |
| `Ctrl+T` | Rename the selected note |
| `Ctrl+D` | Move the selected note to `.trash/` after confirming with `y` |
| `Ctrl+S` | Save now (also overwrites a file changed on disk) |
| `Ctrl+Q` | Save and quit |
| `Ctrl+H` | Show or hide the shortcut list |
| `Esc` | Switch focus (vaults → list → editor); close a prompt or popup |
| `↑` `↓` | Move in the vault list or the folder/note list |
| `Enter` / `→` | Open a note, expand a folder, or switch to the selected vault |
| `←` | Collapse an open folder; otherwise move to the parent folder |
| `r` | Refresh the list |
| Mouse | Click to select or place the cursor; wheel to scroll |

Shortcut letters are matched as English keys. With a Korean input method on, a letter key may not match; switch to English input first.

## Saving

- A note is saved 0.8 seconds after the last keystroke.
- Saving writes a temporary file and renames it, so a note is not left half-written.
- If a save fails, the status line shows the error and the save is retried every 3 seconds.
- `Ctrl+Q` with a failed save shows a warning; press `Ctrl+Q` again to quit without saving.
- If the file changed on disk after it was opened (Obsidian, a sync client), autosave stops and a warning is shown. `Ctrl+S` overwrites the file with your text.
- Deleted notes go to `.trash/` under the list root. Names are kept unique with a UTC timestamp (`name.20261112-055806.md`). There is no restore screen; move the file back yourself.

## Language

The UI supports en, ko, ru, fr, es, de, ja. The language is chosen in this order:

1. `HERDR_OBSIDIAN_LANG` environment variable
2. `lang` in `config.toml`
3. OS language
4. `en`

A message missing in the chosen language falls back to English. The ru, fr, es, de, and ja translations are drafts that have not been reviewed by native speakers. See [i18n.md](i18n.md).

## Requirements

- Windows, Linux, or macOS (developed and tested on Windows)
- herdr 0.9.0 or later
- Rust (stable), to build

## Development

```sh
cargo test
cargo clippy --all-targets
cargo fmt --check
```

## Build and run

```sh
cargo build --release
target/release/herdr-obsidian "C:\work\obsidian\herdr_test"
```

Run the binary in a herdr pane, or install the plugin with `herdr plugin install .` after the build (`herdr-plugin.toml` in the repo root).
