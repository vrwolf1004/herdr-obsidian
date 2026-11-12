# Usage

> Status: planned. Commands and screens below describe the target behavior.

## Vault

Set the vault folder to the Obsidian vault you want to browse. Any folder of `.md` files works, so Google Drive or other sync folders can be used. The config format and location are not decided yet.

## Layout

- Side pane: folders and notes of the vault, as a tree.
- Main pane: the selected note.

## Reading

Click (or select and press Enter) a note in the list to show its Markdown.

## Editing

Edit the note in place. Changes are saved automatically a short time after you stop typing. The save timing and conflict handling (when the file changed outside the plugin) are not decided yet.

## Search

Search the text of notes in the vault. Result format is not decided yet.

## Test tab

A herdr tab is provided for trying the plugin. Open it to see the current build running against a sample vault.

## Limits

- Only Markdown (`.md`) files are read and written.
- Nothing is uploaded; the plugin only touches files in the vault folder.
