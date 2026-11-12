use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};
use ratatui::Frame;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::i18n::t;
use crate::{App, Focus, PromptKind};

const RIGHT_WIDTH: u16 = 48;

pub fn draw(f: &mut Frame, app: &mut App) {
    let [main, status] =
        Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(f.area());
    let [left, right] =
        Layout::horizontal([Constraint::Min(20), Constraint::Length(RIGHT_WIDTH)]).areas(main);
    // Vault list on top (grows with the list, max 10 rows), folder/note tree below.
    let panel_height = (app.vaults.len() as u16 + 2).clamp(3, 10).min(right.height);
    let [vault_area, tree_area] =
        Layout::vertical([Constraint::Length(panel_height), Constraint::Min(1)]).areas(right);
    draw_editor(f, app, left);
    draw_vaults(f, app, vault_area);
    draw_tree(f, app, tree_area);
    draw_status(f, app, status);
    if app.help_open {
        draw_help(f, f.area());
    }
}

/// Bottom line: message or prompt on the left, the Ctrl+H hint pinned to the right edge.
/// The hint has its own cell range, so long messages and prompts never push it out.
fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let hint = t("hint.help", &[]);
    let hint_width = UnicodeWidthStr::width(hint.as_str()) as u16;
    let [left, hint_area] =
        Layout::horizontal([Constraint::Min(1), Constraint::Length(hint_width)]).areas(area);
    let reversed = Style::new().add_modifier(Modifier::REVERSED);
    f.render_widget(
        Paragraph::new(hint).style(reversed.add_modifier(Modifier::BOLD)),
        hint_area,
    );

    let max = left.width as usize;
    match &app.prompt {
        Some(prompt) => {
            // Drawn last so the cursor lands in the prompt, not in the editor.
            // Narrow: keep the end of the line (where typing happens) visible.
            let text = format!("{}{}", prompt_label(&prompt.kind), prompt.input);
            let shown = tail_within(&text, max.saturating_sub(1));
            let cursor_x = left.x + UnicodeWidthStr::width(shown.as_str()) as u16;
            f.render_widget(Paragraph::new(shown).style(reversed), left);
            f.set_cursor_position((cursor_x.min(left.right().saturating_sub(1)), left.y));
        }
        None => {
            // Narrow: swap the default hint list for a shorter one.
            let default = t("status.default", &[]);
            let message = if app.status == default && UnicodeWidthStr::width(default.as_str()) > max
            {
                t("status.short", &[])
            } else {
                app.status.clone()
            };
            f.render_widget(Paragraph::new(message).style(reversed), left);
        }
    }
}

/// Longest suffix of `text` that fits in `max_width` display cells.
fn tail_within(text: &str, max_width: usize) -> String {
    let mut used = 0;
    let mut start = text.len();
    for (i, c) in text.char_indices().rev() {
        let w = UnicodeWidthChar::width(c).unwrap_or(0);
        if used + w > max_width {
            break;
        }
        used += w;
        start = i;
    }
    text[start..].to_string()
}

/// One line of the shortcut popup. Key names are shown as written; words are translated.
enum HelpRow {
    Blank,
    Section(&'static str),
    /// Keys as written (e.g. "Ctrl+O"), description key.
    Key(&'static str, &'static str),
    /// Label key, description key.
    Named(&'static str, &'static str),
}

/// Shortcut list. Only keys that are implemented.
const HELP_ROWS: &[HelpRow] = &[
    HelpRow::Section("help.section.global"),
    HelpRow::Key("Ctrl+O", "help.desc.open_folder"),
    HelpRow::Key("Ctrl+N", "help.desc.new_note"),
    HelpRow::Key("Ctrl+G", "help.desc.new_folder"),
    HelpRow::Key("Ctrl+T", "help.desc.rename"),
    HelpRow::Key("Ctrl+D", "help.desc.trash"),
    HelpRow::Key("Ctrl+S", "help.desc.save"),
    HelpRow::Key("Ctrl+P", "help.desc.preview"),
    HelpRow::Key("Ctrl+Q", "help.desc.quit"),
    HelpRow::Key("Ctrl+H", "help.desc.help"),
    HelpRow::Key("Esc", "help.desc.focus"),
    HelpRow::Blank,
    HelpRow::Section("help.section.vaults"),
    HelpRow::Key("↑ ↓", "help.desc.vault_move"),
    HelpRow::Key("Enter", "help.desc.vault_switch"),
    HelpRow::Key("Delete", "help.desc.vault_remove"),
    HelpRow::Key("Ctrl+E", "help.desc.add_vault"),
    HelpRow::Blank,
    HelpRow::Section("help.section.tree"),
    HelpRow::Key("↑ ↓", "help.desc.tree_move"),
    HelpRow::Key("Enter / →", "help.desc.tree_open"),
    HelpRow::Key("←", "help.desc.tree_left"),
    HelpRow::Key("r", "help.desc.refresh"),
    HelpRow::Key("Tab", "help.desc.tab_editor"),
    HelpRow::Blank,
    HelpRow::Section("help.section.editor"),
    HelpRow::Named("help.label.typing", "help.desc.typing"),
    HelpRow::Key("Enter / Backspace / Delete", "help.desc.edit_keys"),
    HelpRow::Named("help.label.arrows", "help.desc.cursor"),
    HelpRow::Named("help.label.mouse", "help.desc.mouse"),
    HelpRow::Blank,
    HelpRow::Section("help.section.prompt"),
    HelpRow::Key("Enter", "help.desc.confirm"),
    HelpRow::Key("Esc", "help.desc.cancel"),
];

fn help_line(label: String, desc: String) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<28}"), Style::new().fg(Color::Yellow)),
        Span::raw(desc),
    ])
}

fn draw_help(f: &mut Frame, area: Rect) {
    let width = area.width.min(72);
    let height = (HELP_ROWS.len() as u16 + 2).min(area.height);
    let rect = Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    );
    let heading = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    let lines: Vec<Line> = HELP_ROWS
        .iter()
        .map(|row| match row {
            HelpRow::Blank => Line::from(""),
            HelpRow::Section(key) => Line::from(Span::styled(t(key, &[]), heading)),
            HelpRow::Key(keys, desc) => help_line(keys.to_string(), t(desc, &[])),
            HelpRow::Named(label, desc) => help_line(t(label, &[]), t(desc, &[])),
        })
        .collect();
    let block = Block::bordered()
        .title(t("help.title", &[]))
        .border_style(Style::new().fg(Color::Cyan));
    f.render_widget(Clear, rect);
    f.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false }),
        rect,
    );
}

fn prompt_label(kind: &PromptKind) -> String {
    match kind {
        PromptKind::Folder => t("prompt.folder", &[]),
        PromptKind::NewNote => t("prompt.new_note", &[]),
        PromptKind::Rename(_) => t("prompt.rename", &[]),
        PromptKind::NewFolder(_) => t("prompt.new_folder", &[]),
        PromptKind::AddVault => t("prompt.add_vault", &[]),
    }
}

fn border_style(focused: bool) -> Style {
    if focused {
        Style::new().fg(Color::Cyan)
    } else {
        Style::new().fg(Color::DarkGray)
    }
}

fn draw_vaults(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::bordered()
        .title(t("ui.title.vaults", &[]))
        .border_style(border_style(app.focus == Focus::Vaults));
    let inner = block.inner(area);
    f.render_widget(block, area);
    app.vault_area = inner;

    let height = inner.height as usize;
    if app.vault_selected < app.vault_offset {
        app.vault_offset = app.vault_selected;
    }
    if height > 0 && app.vault_selected >= app.vault_offset + height {
        app.vault_offset = app.vault_selected + 1 - height;
    }

    let focused = app.focus == Focus::Vaults;
    let lines: Vec<Line> = app
        .vaults
        .iter()
        .enumerate()
        .skip(app.vault_offset)
        .take(height)
        .map(|(i, path)| {
            let is_current = i == app.current_vault;
            let marker = if is_current { "● " } else { "  " };
            let missing = if path.is_dir() {
                String::new()
            } else {
                t("ui.missing", &[])
            };
            let mut style = if is_current {
                Style::new().fg(Color::Yellow)
            } else {
                Style::new()
            };
            if i == app.vault_selected && focused {
                style = style.add_modifier(Modifier::REVERSED);
            } else if i == app.vault_selected {
                style = style.add_modifier(Modifier::BOLD);
            }
            Line::from(Span::styled(
                format!("{marker}{}{missing}", path.display()),
                style,
            ))
        })
        .collect();
    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_tree(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::bordered()
        .title(format!(" {} ", app.vault.display()))
        .border_style(border_style(app.focus == Focus::Tree));
    let inner = block.inner(area);
    f.render_widget(block, area);
    app.tree_area = inner;

    let height = inner.height as usize;
    if app.selected < app.tree_offset {
        app.tree_offset = app.selected;
    }
    if height > 0 && app.selected >= app.tree_offset + height {
        app.tree_offset = app.selected + 1 - height;
    }

    let focused = app.focus == Focus::Tree;
    let open_path = app.editor.as_ref().map(|e| e.path.clone());
    let lines: Vec<Line> = app
        .tree
        .rows
        .iter()
        .enumerate()
        .skip(app.tree_offset)
        .take(height)
        .map(|(i, row)| {
            let marker = match (row.is_dir, app.tree.expanded.contains(&row.path)) {
                (true, true) => "▾ ",
                (true, false) => "▸ ",
                (false, _) => "  ",
            };
            let mut style = if row.is_dir {
                Style::new().fg(Color::Cyan)
            } else if open_path.as_ref() == Some(&row.path) {
                Style::new().fg(Color::Yellow)
            } else {
                Style::new()
            };
            if i == app.selected && focused {
                style = style.add_modifier(Modifier::REVERSED);
            } else if i == app.selected {
                style = style.add_modifier(Modifier::BOLD);
            }
            let text = format!("{}{marker}{}", "  ".repeat(row.depth), row.name);
            Line::from(Span::styled(text, style))
        })
        .collect();
    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_editor(f: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == Focus::Editor;
    let title = match &app.editor {
        Some(ed) => {
            let name = ed
                .path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let preview = if app.preview {
                t("ui.title.preview", &[])
            } else {
                String::new()
            };
            format!(" {name}{}{preview} ", if ed.dirty { " ●" } else { "" })
        }
        None => t("ui.title.no_note", &[]),
    };
    let block = Block::bordered()
        .title(title)
        .border_style(border_style(focused));
    let inner = block.inner(area);
    f.render_widget(block, area);
    app.editor_area = inner;

    let preview = app.preview;
    let Some(ed) = app.editor.as_mut() else {
        return;
    };
    let height = inner.height as usize;
    let width = inner.width as usize;

    // Keep the cursor inside the viewport.
    let cursor_x: usize = UnicodeWidthStr::width(
        ed.lines[ed.row]
            .chars()
            .take(ed.col)
            .collect::<String>()
            .as_str(),
    );
    if ed.row < ed.scroll_y {
        ed.scroll_y = ed.row;
    }
    if height > 0 && ed.row >= ed.scroll_y + height {
        ed.scroll_y = ed.row + 1 - height;
    }
    if cursor_x < ed.scroll_x {
        ed.scroll_x = cursor_x;
    }
    if width > 0 && cursor_x >= ed.scroll_x + width {
        ed.scroll_x = cursor_x + 1 - width;
    }

    let mut in_fence = false;
    let lines: Vec<Line> = if preview {
        ed.lines
            .iter()
            .map(|l| md_preview_line(l, &mut in_fence))
            .collect()
    } else {
        ed.lines.iter().map(|l| md_line(l, &mut in_fence)).collect()
    };
    f.render_widget(
        Paragraph::new(Text::from(lines)).scroll((ed.scroll_y as u16, ed.scroll_x as u16)),
        inner,
    );

    if focused && !preview && height > 0 && !app.help_open {
        let x = inner.x + (cursor_x - ed.scroll_x) as u16;
        let y = inner.y + (ed.row - ed.scroll_y) as u16;
        f.set_cursor_position((x, y));
    }
}

/// Light markdown styling by line prefix: headings, quotes, list items, fenced code.
fn md_line(text: &str, in_fence: &mut bool) -> Line<'static> {
    let trimmed = text.trim_start();
    let hashes = trimmed.len() - trimmed.trim_start_matches('#').len();
    let is_heading = (1..=6).contains(&hashes) && trimmed[hashes..].starts_with(' ');
    let is_ordered = trimmed
        .split_once(". ")
        .is_some_and(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()));

    let style = if trimmed.starts_with("```") {
        *in_fence = !*in_fence;
        Style::new().fg(Color::DarkGray)
    } else if *in_fence {
        Style::new().fg(Color::Green)
    } else if is_heading {
        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else if trimmed.starts_with('>') {
        Style::new()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC)
    } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") || is_ordered {
        Style::new().fg(Color::Yellow)
    } else {
        Style::new()
    };
    Line::from(Span::styled(text.to_string(), style))
}

/// Preview of one source line: markers hidden, styles applied. Keeps one output line per source line.
fn md_preview_line(text: &str, in_fence: &mut bool) -> Line<'static> {
    let trimmed = text.trim_start();
    if trimmed.starts_with("```") {
        *in_fence = !*in_fence;
        return Line::default();
    }
    if *in_fence {
        return Line::from(Span::styled(
            text.to_string(),
            Style::new().fg(Color::Green),
        ));
    }
    let indent = &text[..text.len() - trimmed.len()];
    let hashes = trimmed.len() - trimmed.trim_start_matches('#').len();
    if (1..=6).contains(&hashes) && trimmed[hashes..].starts_with(' ') {
        let style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);
        return Line::from(md_inline(&trimmed[hashes + 1..], style));
    }
    if let Some(body) = trimmed.strip_prefix('>') {
        let style = Style::new()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC);
        return Line::from(md_inline(body.trim_start(), style));
    }
    if let Some(rest) = trimmed.strip_prefix("- ").or(trimmed.strip_prefix("* ")) {
        let mut spans = vec![Span::raw(format!("{indent}• "))];
        spans.extend(md_inline(rest, Style::new()));
        return Line::from(spans);
    }
    Line::from(md_inline(text, Style::new()))
}

/// Split inline markdown into styled spans: `code`, **bold**, *italic*, [text](url) (url hidden).
fn md_inline(text: &str, base: Style) -> Vec<Span<'static>> {
    let chars: Vec<char> = text.chars().collect();
    let mut spans = Vec::new();
    let mut plain = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let styled = match c {
            '`' => find(&chars, i + 1, &['`']).map(|j| {
                let span = Span::styled(collect(&chars[i + 1..j]), base.fg(Color::Magenta));
                (span, j + 1)
            }),
            '*' | '_' if chars.get(i + 1) == Some(&c) => find(&chars, i + 2, &[c, c]).map(|j| {
                let span =
                    Span::styled(collect(&chars[i + 2..j]), base.add_modifier(Modifier::BOLD));
                (span, j + 2)
            }),
            '*' | '_' if chars.get(i + 1).is_some_and(|n| !n.is_whitespace()) => {
                find(&chars, i + 1, &[c])
                    .filter(|&j| !chars[j - 1].is_whitespace())
                    .map(|j| {
                        let span = Span::styled(
                            collect(&chars[i + 1..j]),
                            base.add_modifier(Modifier::ITALIC),
                        );
                        (span, j + 1)
                    })
            }
            '[' => find(&chars, i + 1, &[']']).and_then(|j| {
                if chars.get(j + 1) != Some(&'(') {
                    return None;
                }
                let k = find(&chars, j + 2, &[')'])?;
                let style = base.fg(Color::Cyan).add_modifier(Modifier::UNDERLINED);
                Some((Span::styled(collect(&chars[i + 1..j]), style), k + 1))
            }),
            _ => None,
        };
        match styled {
            Some((span, next)) => {
                if !plain.is_empty() {
                    spans.push(Span::styled(std::mem::take(&mut plain), base));
                }
                spans.push(span);
                i = next;
            }
            None => {
                plain.push(c);
                i += 1;
            }
        }
    }
    if !plain.is_empty() {
        spans.push(Span::styled(plain, base));
    }
    spans
}

/// Index of the first `pat` at or after `from`.
fn find(chars: &[char], from: usize, pat: &[char]) -> Option<usize> {
    (from..chars.len()).find(|&j| chars[j..].starts_with(pat))
}

fn collect(chars: &[char]) -> String {
    chars.iter().collect()
}

#[cfg(test)]
mod tests {
    use super::{md_inline, md_preview_line, tail_within};
    use ratatui::style::Modifier;

    fn text_of(spans: &[ratatui::text::Span]) -> String {
        spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn inline_hides_markers_and_styles_spans() {
        let spans = md_inline("a **b** `c` *d* [e](f) g", Default::default());
        assert_eq!(text_of(&spans), "a b c d e g");
        assert!(spans[1].style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn inline_leaves_unclosed_markers_as_text() {
        let spans = md_inline("2 * 3 and `x", Default::default());
        assert_eq!(text_of(&spans), "2 * 3 and `x");
    }

    #[test]
    fn preview_hides_heading_and_fence_markers() {
        let mut in_fence = false;
        assert_eq!(
            text_of(&md_preview_line("## Title", &mut in_fence).spans),
            "Title"
        );
        assert!(md_preview_line("```", &mut in_fence).spans.is_empty());
        assert_eq!(
            text_of(&md_preview_line("let x", &mut in_fence).spans),
            "let x"
        );
        assert!(md_preview_line("```", &mut in_fence).spans.is_empty());
        assert!(!in_fence);
    }

    #[test]
    fn preview_bullets_use_dot() {
        let mut in_fence = false;
        assert_eq!(
            text_of(&md_preview_line("  - item", &mut in_fence).spans),
            "  • item"
        );
    }

    #[test]
    fn tail_keeps_end_and_counts_wide_chars() {
        assert_eq!(tail_within("abcdef", 3), "def");
        // Two wide characters take 4 cells; 4 cells keeps the last two.
        assert_eq!(tail_within("가나다", 4).chars().count(), 2);
        assert_eq!(tail_within("abc", 10), "abc");
        assert_eq!(tail_within("abc", 0), "");
    }

    #[test]
    fn every_help_row_has_a_translated_text() {
        use super::HELP_ROWS;
        // Rows that show text must not render as their own key name.
        for row in HELP_ROWS {
            if let super::HelpRow::Key(_, desc) = row {
                assert_ne!(super::t(desc, &[]), *desc, "missing {desc}");
            }
        }
    }
}
