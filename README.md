# herdr-obsidian

[**English**](README.md) | [한국어](README.ko.md)

UI language: Korean

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

There is no `herdr-plugin.toml` yet, so `herdr plugin install` does not work. Build it and run it in a herdr pane:

```sh
git clone https://github.com/vrwolf1004/herdr-obsidian.git
cd herdr-obsidian
cargo build --release
target/release/herdr-obsidian "C:\work\obsidian\herdr_test"
```

## Quick start

- `Ctrl+O`: open a folder in the vault.
- `Ctrl+N` / `Ctrl+G`: create a note / folder.
- `Ctrl+H`: show all shortcuts.

## Documentation

- [Usage: vault list, shortcuts, saving (English)](docs/usage.md)
- [사용법 (한국어)](docs/usage.ko.md)
- [Release notes](docs/release-notes.md)
- [TODO](docs/todo.md)

## License

MIT

## Support the project

If this project is useful to you, consider buying the developer a coffee.

[![Support on Ko-fi](https://img.shields.io/badge/Ko--fi-Support-FF5E5B?style=for-the-badge&logo=buymeacoffee&logoColor=white)](https://ko-fi.com/dev_ohyt)
