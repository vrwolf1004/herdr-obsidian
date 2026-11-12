use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};
use ratatui::Frame;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

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

pub const DEFAULT_STATUS: &str =
    "Ctrl+O 폴더 · Ctrl+N 새 노트 · Ctrl+T 이름 변경 · Ctrl+D 삭제 · Ctrl+S 저장 · Ctrl+Q 종료";
const SHORT_STATUS: &str = "Ctrl+O 폴더 · Ctrl+N 노트 · Ctrl+S 저장 · Ctrl+Q 종료";
const HINT: &str = " Ctrl+H 단축키 ";

/// Bottom line: message or prompt on the left, the Ctrl+H hint pinned to the right edge.
/// The hint has its own cell range, so long messages and prompts never push it out.
fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let hint_width = UnicodeWidthStr::width(HINT) as u16;
    let [left, hint] =
        Layout::horizontal([Constraint::Min(1), Constraint::Length(hint_width)]).areas(area);
    let reversed = Style::new().add_modifier(Modifier::REVERSED);
    f.render_widget(
        Paragraph::new(HINT).style(reversed.add_modifier(Modifier::BOLD)),
        hint,
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
            let message =
                if app.status == DEFAULT_STATUS && UnicodeWidthStr::width(DEFAULT_STATUS) > max {
                    SHORT_STATUS
                } else {
                    app.status.as_str()
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

/// Shortcut list. Only keys that are implemented; an empty key marks a section title.
const HELP_ROWS: &[(&str, &str)] = &[
    ("", "전역"),
    ("Ctrl+O", "폴더 열기 (볼트 기준 상대 경로 또는 절대 경로)"),
    ("Ctrl+N", "현재 목록 루트에 새 노트 만들기 (.md 자동)"),
    ("Ctrl+G", "선택한 폴더 안에 새 폴더 만들기"),
    ("Ctrl+T", "선택한 파일 이름 변경"),
    ("Ctrl+D", "선택한 파일 삭제 (y 확인)"),
    ("Ctrl+S", "저장"),
    ("Ctrl+Q", "저장 후 종료"),
    ("Ctrl+H", "이 단축키 목록 열기/닫기"),
    ("Esc", "포커스 전환 (볼트 → 목록 → 편집기), 입력·팝업 닫기"),
    ("", ""),
    ("", "볼트 목록"),
    ("↑ ↓", "볼트 선택"),
    ("Enter", "선택한 볼트로 전환"),
    ("", ""),
    ("", "폴더·노트 목록"),
    ("↑ ↓", "이동 (맨 위에서 ↑ 는 볼트 목록으로)"),
    ("Enter / →", "노트 열기, 폴더 펼치기/접기"),
    ("←", "펼쳐진 폴더 접기, 아니면 상위 폴더로"),
    ("r", "목록 새로고침"),
    ("Tab", "열린 노트를 편집기로 이동"),
    ("", ""),
    ("", "편집기"),
    ("글자 입력", "입력 (Tab 은 공백 4칸)"),
    ("Enter / Backspace / Delete", "줄바꿈 / 글자 삭제"),
    ("방향키 · Home · End", "커서 이동 (PgUp / PgDn 은 10줄씩)"),
    ("마우스", "클릭으로 커서·선택, 휠로 스크롤"),
    ("", ""),
    ("", "입력줄 (Ctrl+O/N/G/T)"),
    ("Enter", "확인"),
    ("Esc", "취소"),
];

fn draw_help(f: &mut Frame, area: Rect) {
    let width = area.width.min(72);
    let height = (HELP_ROWS.len() as u16 + 2).min(area.height);
    let rect = Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    );
    let lines: Vec<Line> = HELP_ROWS
        .iter()
        .map(|(key, desc)| {
            if key.is_empty() {
                Line::from(Span::styled(
                    desc.to_string(),
                    Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(vec![
                    Span::styled(format!("{key:<28}"), Style::new().fg(Color::Yellow)),
                    Span::raw(desc.to_string()),
                ])
            }
        })
        .collect();
    let block = Block::bordered()
        .title(" 단축키 (Esc / Ctrl+H 닫기) ")
        .border_style(Style::new().fg(Color::Cyan));
    f.render_widget(Clear, rect);
    f.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false }),
        rect,
    );
}

fn prompt_label(kind: &PromptKind) -> &'static str {
    match kind {
        PromptKind::Folder => " 폴더 경로 (볼트 기준, 비우면 볼트 루트, Esc 취소) › ",
        PromptKind::NewNote => " 새 노트 이름 (.md 자동, Esc 취소) › ",
        PromptKind::Rename(_) => " 새 이름 (Esc 취소) › ",
        PromptKind::NewFolder(_) => " 새 폴더 이름 (Esc 취소) › ",
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
        .title(" 볼트 목록 (↑↓ Enter 전환) ")
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
            let missing = if path.is_dir() { "" } else { " (없음)" };
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
            format!(" {name}{} ", if ed.dirty { " ●" } else { "" })
        }
        None => " 노트를 선택하세요 ".to_string(),
    };
    let block = Block::bordered()
        .title(title)
        .border_style(border_style(focused));
    let inner = block.inner(area);
    f.render_widget(block, area);
    app.editor_area = inner;

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
    let lines: Vec<Line> = ed.lines.iter().map(|l| md_line(l, &mut in_fence)).collect();
    f.render_widget(
        Paragraph::new(Text::from(lines)).scroll((ed.scroll_y as u16, ed.scroll_x as u16)),
        inner,
    );

    if focused && height > 0 && !app.help_open {
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

#[cfg(test)]
mod tests {
    use super::tail_within;

    #[test]
    fn tail_keeps_end_and_counts_wide_chars() {
        assert_eq!(tail_within("abcdef", 3), "def");
        // "가나다" is 6 cells; 4 cells keeps "나다".
        assert_eq!(tail_within("가나다", 4), "나다");
        assert_eq!(tail_within("abc", 10), "abc");
        assert_eq!(tail_within("abc", 0), "");
    }
}
