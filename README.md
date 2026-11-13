# herdr-obsidian

[**English**](README.md) | [한국어](README.ko.md)

UI languages: English · Korean · Russian · French · Spanish · German · Japanese

Browse and edit an [Obsidian](https://obsidian.md) vault from one herdr pane. Notes are plain Markdown files, so any folder works, including one synced by Google Drive for desktop.

## Dashboard design

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

Editor on the left, vault list and folder/note list on the right, status line at the bottom. This is a full-screen terminal app in a pane, not a block in the Agents sidebar.

## Installation

Build first, then install the plugin from the folder (`herdr-plugin.toml` is in the repo root):

```sh
git clone https://github.com/vrwolf1004/herdr-obsidian.git
cd herdr-obsidian
cargo build --release
herdr plugin link .
```

Or run the binary directly in a herdr pane:

```sh
target/release/herdr-obsidian "C:\work\obsidian\herdr_test"
```

## Actions

| Action | What it does |
| --- | --- |
| `open` | Open the vault app in a pane. Without a vault argument it uses the `vaults` list in `config.toml` |

On Windows, append `-windows` to the action ID. In the app, `Ctrl+O` opens a folder in the vault, `Ctrl+N` / `Ctrl+G` create a note / folder, and `Ctrl+H` shows all shortcuts. See the [usage guide](docs/usage.md).

## Requirements

- Windows, Linux, or macOS (developed and tested on Windows)
- herdr 0.9.0 or later
- Rust (stable), to build

## Documentation

- [Usage: vault list, shortcuts, saving (English)](docs/usage.md)
- [사용법 (한국어)](docs/usage.ko.md)
- [Release notes](docs/release-notes.md)
- [TODO](docs/todo.md)
- [Additional sections](docs/readme-extras.md): sections moved out of this README

## Support the project

If this project is useful to you, consider buying the developer a coffee.

[![Support on Ko-fi](https://img.shields.io/badge/Ko--fi-Support-FF5E5B?style=for-the-badge&logo=buymeacoffee&logoColor=white)](https://ko-fi.com/dev_ohyt)
