use crate::{
    app::command_registry::CommandRegistry,
    app::{AppState, Screen},
    fs::{EntryKind, FileEntry},
    layout::{LayoutMetrics, text::pad_or_truncate},
    theme::schema::ThemeRole,
};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use unicode_segmentation::UnicodeSegmentation;

mod palette;

pub fn render(frame: &mut Frame<'_>, state: &AppState, metrics: &LayoutMetrics) {
    palette::set_theme(&state.theme);
    if metrics.too_small {
        render_too_small(frame, metrics.viewport);
        return;
    }

    if state.screen == Screen::Viewer {
        render_viewer(frame, metrics.viewport, state);
        return;
    }
    if state.screen == Screen::GitDiff {
        render_git_diff(frame, metrics.viewport, state);
        return;
    }

    render_main(frame, state, metrics);
    match state.screen {
        Screen::Help => render_help(
            frame,
            metrics.viewport,
            &state.registry,
            &state.plugin_commands,
        ),
        Screen::QuitConfirm => render_quit_confirmation(frame, metrics.viewport),
        Screen::InputDialog => render_input_dialog(frame, state, metrics.viewport),
        Screen::ConfirmDialog => render_confirm_dialog(frame, state, metrics.viewport),
        Screen::Viewer | Screen::GitDiff => {
            unreachable!("full-screen document renders before the main screen")
        }
        Screen::Editor => render_editor(frame, state, metrics.viewport),
        Screen::Progress => render_progress(frame, state, metrics.viewport),
        Screen::DrivePicker => render_drive_picker(frame, state, metrics.viewport),
        Screen::ConflictDialog => render_conflict_dialog(frame, state, metrics.viewport),
        Screen::Mcd => render_mcd(frame, state, metrics.viewport),
        Screen::Qcd => render_qcd(frame, state, metrics.viewport),
        Screen::Menu => render_menu(frame, state, metrics.viewport),
        Screen::Settings => render_settings(frame, state, metrics.viewport),
        Screen::GitStatus => render_git_status(frame, state, metrics.viewport),
        Screen::Main => {}
    }
}

fn render_git_status(frame: &mut Frame<'_>, state: &AppState, viewport: Rect) {
    frame.render_widget(Clear, viewport);
    let Some(view) = &state.git_status_view else {
        return;
    };
    let rows: Vec<_> = view
        .rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            Line::raw(format!(
                "{}{} {} {}",
                if index == view.selected { ">" } else { " " },
                if view
                    .marked
                    .contains(&row.path.as_path().display().to_string())
                {
                    "*"
                } else {
                    " "
                },
                crate::plugins::git::decoration::prefix(row.status),
                row.path.as_path().display()
            ))
        })
        .collect();
    frame.render_widget(
        Paragraph::new(rows).block(
            dialog_block(" Git Status ").title_bottom(
                "Enter/F3 Diff  F5 Stage  F6 Unstage  Space Mark  R Refresh  Esc Close",
            ),
        ),
        viewport,
    );
}

fn render_git_diff(frame: &mut Frame<'_>, viewport: Rect, state: &AppState) {
    frame.render_widget(Clear, viewport);
    let Some((path, viewer)) = &state.git_diff else {
        return;
    };
    render_document_viewer(
        frame,
        viewport,
        viewer,
        &format!(" Git Diff: {} ", path.display()),
        "Esc Back  Up/Down Scroll  PgUp/PgDn Page  Ctrl+F Find  F3 Next",
    );
}

fn render_menu(frame: &mut Frame<'_>, state: &AppState, viewport: Rect) {
    const CATEGORIES: [&str; 6] = ["File", "View", "Directory", "Tools", "Options", "Quit"];
    const ITEMS: [&[&str]; 6] = [
        &["Rename", "Copy", "Move", "Move to Trash"],
        &[
            "Toggle Short/Long",
            "Next Sort",
            "Sort Direction",
            "Hidden Files",
        ],
        &["Make Directory", "MCD Tree", "QCD Favorites"],
        &["View File", "Edit File", "Drives"],
        &["Settings"],
        &["Quit Mdir4"],
    ];
    let area = centered_rect(64, 14, viewport);
    frame.render_widget(Clear, area);
    let mut lines = vec![Line::raw(
        CATEGORIES
            .iter()
            .enumerate()
            .map(|(index, value)| {
                if index == state.menu_category {
                    format!("[{value}]")
                } else {
                    value.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("  "),
    )];
    lines.push(Line::raw(""));
    lines.extend(
        ITEMS[state.menu_category]
            .iter()
            .enumerate()
            .map(|(index, value)| {
                Line::raw(format!(
                    "{} {value}",
                    if index == state.menu_item { ">" } else { " " }
                ))
            }),
    );
    lines.push(Line::raw(""));
    lines.push(Line::raw("Arrows Navigate  Enter Select  Esc Close"));
    frame.render_widget(
        Paragraph::new(lines).block(dialog_block(" Mdir4 Menu ")),
        area,
    );
}

fn render_settings(frame: &mut Frame<'_>, state: &AppState, viewport: Rect) {
    let area = centered_rect(64, 16, viewport);
    frame.render_widget(Clear, area);
    let Some(draft) = state.settings_preview.as_ref() else {
        return;
    };
    let values = [
        format!("View: {}", if draft.long_view { "Long" } else { "Short" }),
        format!("Theme: {}", draft.theme),
        format!(
            "Column count: {}",
            draft
                .column_count
                .map_or("Auto".to_string(), |value| value.to_string())
        ),
        format!(
            "Column width: {}",
            draft
                .column_width
                .map_or("Auto".to_string(), |value| value.to_string())
        ),
        format!("Sort key: {:?}", draft.sort_key),
        format!("Sort direction: {:?}", draft.sort_direction),
        format!(
            "Show hidden: {}",
            if draft.show_hidden { "Yes" } else { "No" }
        ),
        format!(
            "Keymap: {}",
            if draft.use_custom_keymap {
                "Custom"
            } else {
                "Default"
            }
        ),
    ];
    let mut lines: Vec<_> = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            Line::raw(format!(
                "{} {value}",
                if index == state.settings_cursor {
                    ">"
                } else {
                    " "
                }
            ))
        })
        .collect();
    lines.push(Line::raw(""));
    lines.push(Line::raw("Left/Right Change  Enter Apply  Esc Cancel"));
    frame.render_widget(
        Paragraph::new(lines).block(dialog_block(" Settings Preview ")),
        area,
    );
}

fn render_qcd(frame: &mut Frame<'_>, state: &AppState, viewport: Rect) {
    let area = centered_rect(
        viewport.width.saturating_sub(8).min(72),
        viewport.height.saturating_sub(6).min(20),
        viewport,
    );
    frame.render_widget(Clear, area);
    let mut lines: Vec<Line<'_>> = state
        .qcd
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            Line::raw(format!(
                "{} {:>2}. {:<18} {}",
                if index == state.selected_qcd {
                    ">"
                } else {
                    " "
                },
                index + 1,
                entry.label,
                entry.path.display()
            ))
        })
        .collect();
    if lines.is_empty() {
        lines.push(Line::raw("(no favorite directories)"));
    }
    lines.push(Line::raw(
        "Insert Add  F2 Edit  D Delete  Ctrl+Up/Down Reorder  Enter Open  Esc Close",
    ));
    frame.render_widget(
        Paragraph::new(lines).block(dialog_block(" QCD Favorites ")),
        area,
    );
}

fn render_mcd(frame: &mut Frame<'_>, state: &AppState, viewport: Rect) {
    let Some(tree) = &state.mcd else { return };
    frame.render_widget(Clear, viewport);
    frame.render_widget(
        Block::default().style(palette::role(ThemeRole::McdBackground)),
        viewport,
    );
    if tree.is_loading_path(&state.current_path) {
        let message = centered_rect(viewport.width.saturating_sub(8).min(48), 3, viewport);
        frame.render_widget(
            Paragraph::new("Loading current directory tree…")
                .alignment(Alignment::Center)
                .style(palette::role(ThemeRole::McdBackground)),
            message,
        );
        frame.render_widget(
            Paragraph::new("Esc Cancel").style(palette::role(ThemeRole::McdBackground)),
            Rect::new(
                viewport.x,
                viewport.y.saturating_add(viewport.height.saturating_sub(1)),
                viewport.width,
                1,
            ),
        );
        return;
    }
    let header = Rect::new(viewport.x, viewport.y, viewport.width, 1);
    let body = Rect::new(
        viewport.x,
        viewport.y.saturating_add(1),
        viewport.width,
        viewport.height.saturating_sub(2),
    );
    let footer = Rect::new(
        viewport.x,
        viewport.y.saturating_add(viewport.height.saturating_sub(1)),
        viewport.width,
        1,
    );
    let all_rows = tree.visible_rows();
    let (start, rows) = tree.visible_window(body.height as usize);
    let end = start + rows.len();
    let title = format!(
        "Mdir4 Change Directory  [{}-{}/{}]",
        if all_rows.is_empty() { 0 } else { start + 1 },
        end,
        all_rows.len()
    );
    frame.render_widget(
        Paragraph::new(title).style(palette::role(ThemeRole::McdBackground)),
        header,
    );
    let mut lines = Vec::with_capacity(rows.len());
    for (index, row) in rows.iter().enumerate() {
        let node = tree.node(row.id).unwrap();
        let prefix = row
            .connector_continues
            .iter()
            .take(row.connector_continues.len().saturating_sub(1))
            .map(|continues| if *continues { "│  " } else { "   " })
            .collect::<String>();
        let branch = if node.depth == 0 {
            ""
        } else if row.connector_continues.last().copied().unwrap_or(false) {
            "├─ "
        } else {
            "└─ "
        };
        let marker = match node.state {
            crate::mcd::tree::LoadState::Loading => " …",
            crate::mcd::tree::LoadState::Error(_) => " !",
            _ => "",
        };
        let name = node
            .path
            .file_name()
            .unwrap_or(node.path.as_os_str())
            .to_string_lossy();
        let line = format!(
            "{}{}{}{}{}",
            if start + index == tree.selected {
                "> "
            } else {
                "  "
            },
            prefix,
            branch,
            name,
            marker
        );
        lines.push(Line::raw(line));
    }
    frame.render_widget(
        Paragraph::new(lines).style(palette::role(ThemeRole::McdBackground)),
        body,
    );
    let above = if start > 0 { "▲ " } else { "  " };
    let below = if end < all_rows.len() { "▼ " } else { "  " };
    frame.render_widget(
        Paragraph::new(format!(
            "{above}{below}PgUp/PgDn  F2 Rescan  F3 Drives  Enter Open  Esc Cancel"
        ))
        .style(palette::role(ThemeRole::McdBackground)),
        footer,
    );
}

fn render_conflict_dialog(frame: &mut Frame<'_>, state: &AppState, viewport: Rect) {
    let Some((source, target)) = &state.conflict else {
        return;
    };
    let area = centered_rect(viewport.width.saturating_sub(8).min(72), 10, viewport);
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(format!(
            "Source: {}\nTarget: {}\n\nO Overwrite   A Overwrite All\nS Skip        K Skip All\nR Rename      C/Esc Cancel",
            source.display(), target.display()
        )).block(dialog_block(" File Conflict ")),
        area,
    );
}

fn render_drive_picker(frame: &mut Frame<'_>, state: &AppState, viewport: Rect) {
    let body = if state.drives.is_empty() {
        "Loading drives...".to_string()
    } else {
        state
            .drives
            .iter()
            .enumerate()
            .map(|(index, path)| {
                format!(
                    "{} {}",
                    if index == state.selected_drive {
                        ">"
                    } else {
                        " "
                    },
                    path.display()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let area = centered_rect(viewport.width.saturating_sub(8).min(50), 8, viewport);
    frame.render_widget(Clear, area);
    frame.render_widget(Paragraph::new(body).block(dialog_block(" Drives ")), area);
}

fn render_input_dialog(frame: &mut Frame<'_>, state: &AppState, viewport: Rect) {
    let Some(dialog) = &state.input_dialog else {
        return;
    };
    let area = centered_rect(viewport.width.saturating_sub(8).min(72), 8, viewport);
    frame.render_widget(Clear, area);
    let error = dialog.error.as_deref().unwrap_or("");
    frame.render_widget(dialog_block(&format!(" {} ", dialog.title)), area);
    let inner_x = area.x.saturating_add(1);
    let inner_width = area.width.saturating_sub(2);
    frame.render_widget(
        Paragraph::new(dialog.prompt.as_str()),
        Rect::new(inner_x, area.y.saturating_add(1), inner_width, 1),
    );
    let (visible_value, cursor_x) =
        visible_input(&dialog.value, dialog.cursor, inner_width as usize);
    let input_area = Rect::new(inner_x, area.y.saturating_add(3), inner_width, 1);
    frame.render_widget(Paragraph::new(visible_value), input_area);
    frame.render_widget(
        Paragraph::new(error),
        Rect::new(inner_x, area.y.saturating_add(4), inner_width, 1),
    );
    frame.render_widget(
        Paragraph::new("Enter Confirm   Esc Cancel"),
        Rect::new(inner_x, area.y.saturating_add(6), inner_width, 1),
    );
    if inner_width > 0 {
        frame.set_cursor_position((
            input_area.x + cursor_x.min(inner_width.saturating_sub(1)),
            input_area.y,
        ));
    }
}

fn visible_input(value: &str, cursor: usize, max_cells: usize) -> (String, u16) {
    if max_cells == 0 {
        return (String::new(), 0);
    }
    let graphemes = value.graphemes(true).collect::<Vec<_>>();
    let cursor = cursor.min(graphemes.len());
    let cursor_limit = max_cells.saturating_sub(1);
    let mut start = cursor;
    let mut cursor_cells = 0;
    while start > 0 {
        let width = crate::layout::text::cell_width(graphemes[start - 1]);
        if cursor_cells + width > cursor_limit {
            break;
        }
        start -= 1;
        cursor_cells += width;
    }

    let mut visible = String::new();
    let mut used = 0;
    for grapheme in &graphemes[start..] {
        let width = crate::layout::text::cell_width(grapheme);
        if used + width > max_cells {
            break;
        }
        visible.push_str(grapheme);
        used += width;
    }
    (visible, cursor_cells.min(u16::MAX as usize) as u16)
}

fn render_confirm_dialog(frame: &mut Frame<'_>, state: &AppState, viewport: Rect) {
    let Some(dialog) = &state.confirm_dialog else {
        return;
    };
    let area = centered_rect(viewport.width.saturating_sub(8).min(68), 7, viewport);
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(format!(
            "{}\n\nEnter {}   Esc Cancel",
            dialog.message, dialog.confirm_label
        ))
        .alignment(Alignment::Center)
        .block(dialog_block(&format!(" {} ", dialog.title))),
        area,
    );
}

fn render_viewer(frame: &mut Frame<'_>, viewport: Rect, state: &AppState) {
    let area = viewport;
    frame.render_widget(Clear, area);
    let (path, viewer) = match &state.viewer {
        Some(value) => value,
        None => return,
    };
    render_document_viewer(
        frame,
        area,
        viewer,
        &format!(" View: {} ", path.display()),
        "Esc Close  Up/Down Scroll  PgUp/PgDn Page  Ctrl+F Find  F3 Next",
    );
}

fn render_document_viewer(
    frame: &mut Frame<'_>,
    area: Rect,
    viewer: &crate::model::viewer::ViewerState,
    title: &str,
    help: &str,
) {
    use crate::model::viewer::ViewerState;
    let body_height = area.height.saturating_sub(3) as usize;
    let mut lines: Vec<Line<'_>> = match viewer {
        ViewerState::Loading { .. } => vec![Line::raw("Loading...")],
        ViewerState::Binary => vec![Line::raw("Binary file preview is not available.")],
        ViewerState::TooLarge => vec![Line::raw("File is too large to view (maximum 32 MiB).")],
        ViewerState::Error(error) => vec![Line::raw(error)],
        ViewerState::Ready(document) => (document.top_line..document.top_line + body_height)
            .map(|line| {
                Line::raw(pad_or_truncate(
                    document.line(line),
                    area.width.saturating_sub(2) as usize,
                ))
            })
            .collect(),
    };
    lines.push(Line::raw(help));
    frame.render_widget(Paragraph::new(lines).block(dialog_block(title)), area);
}

fn render_editor(frame: &mut Frame<'_>, state: &AppState, viewport: Rect) {
    let area = Rect::new(
        viewport.x + 1,
        viewport.y + 1,
        viewport.width.saturating_sub(2),
        viewport.height.saturating_sub(2),
    );
    frame.render_widget(Clear, area);
    let (path, editor) = match &state.editor {
        Some(value) => value,
        None => return,
    };
    let height = area.height.saturating_sub(3) as usize;
    let width = area.width.saturating_sub(8) as usize;
    let mut lines: Vec<Line<'_>> = editor
        .text()
        .lines()
        .take(height)
        .enumerate()
        .map(|(index, line)| {
            Line::raw(format!(
                "{:>4}  {}",
                index + 1,
                pad_or_truncate(line, width)
            ))
        })
        .collect();
    lines.push(Line::raw(format!(
        "{}  Cursor {}  Ctrl+S Save  Ctrl+Shift+S Save As  Ctrl+Z/Y Undo/Redo  Esc Close",
        if editor.dirty { "Modified" } else { "Saved" },
        editor.cursor_grapheme() + 1
    )));
    frame.render_widget(
        Paragraph::new(lines).block(dialog_block(&format!(" Edit: {} ", path.display()))),
        area,
    );
}

fn render_progress(frame: &mut Frame<'_>, state: &AppState, viewport: Rect) {
    let area = centered_rect(44, 5, viewport);
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(format!(
            "{}\n\nEsc Cancel",
            state.message.as_deref().unwrap_or("Working...")
        ))
        .alignment(Alignment::Center)
        .block(dialog_block(" Progress ")),
        area,
    );
}

fn dialog_block<'a>(title: &'a str) -> Block<'a> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(palette::role(ThemeRole::DialogBorder))
        .style(palette::role(ThemeRole::Dialog))
}

fn render_main(frame: &mut Frame<'_>, state: &AppState, metrics: &LayoutMetrics) {
    let path = state.current_path.to_string_lossy();
    frame.render_widget(
        Paragraph::new(pad_or_truncate(&path, metrics.path_bar.width as usize))
            .style(palette::role(ThemeRole::PathBar)),
        metrics.path_bar,
    );

    let page_start = state
        .selected
        .checked_div(metrics.page_capacity)
        .unwrap_or_default()
        * metrics.page_capacity;
    if state.long_view {
        render_long_view(frame, state, metrics, page_start);
    } else {
        for (column_index, column) in metrics.columns.iter().enumerate() {
            let has_separator = column_index + 1 < metrics.columns.len();
            let content_width = column.width.saturating_sub(u16::from(has_separator));
            let mut lines = Vec::with_capacity(metrics.rows_per_column);
            for row in 0..metrics.rows_per_column {
                let index = page_start + column_index * metrics.rows_per_column + row;
                let Some(entry) = state.entries.get(index) else {
                    lines.push(Line::raw(""));
                    continue;
                };
                let text = format_entry_with_decoration(
                    entry,
                    content_width as usize,
                    state
                        .plugin_decorations
                        .get(&entry.path.display().to_string()),
                );
                lines.push(Line::from(Span::styled(
                    text,
                    palette::entry(
                        entry,
                        index == state.selected,
                        state.marked.contains(&entry.path),
                    ),
                )));
            }
            frame.render_widget(
                Paragraph::new(Text::from(lines)).style(palette::role(ThemeRole::MainBackground)),
                *column,
            );
            if has_separator {
                frame.render_widget(
                    Block::default()
                        .borders(Borders::RIGHT)
                        .border_style(palette::role(ThemeRole::ColumnSeparator)),
                    *column,
                );
            }
        }
    }

    let detail = state
        .selected_entry()
        .map(|entry| {
            let kind = match entry.kind {
                EntryKind::Parent => "UP",
                EntryKind::Directory => "DIR",
                EntryKind::File => "FILE",
                EntryKind::Other => "OTHER",
            };
            format!(
                "{}   {}   {}   {}",
                entry.display_name(),
                kind,
                human_size(entry.size),
                format_attributes(entry)
            )
        })
        .unwrap_or_else(|| "(no items)".to_string());
    render_status_line(
        frame,
        detail,
        metrics.item_detail,
        palette::role(ThemeRole::StatusBar),
    );

    let (files, directories) = state.file_and_directory_count();
    let (marked, marked_bytes) = state.marked_summary();
    let free = state
        .free_space
        .map(human_size)
        .unwrap_or_else(|| "--".into());
    let summary = format!(
        "Files {files}  Dirs {directories}  Selected {marked} / {}  Free {free}  Items {}",
        human_size(marked_bytes),
        state.entries.len().saturating_sub(usize::from(
            state
                .entries
                .first()
                .is_some_and(|entry| entry.kind == EntryKind::Parent)
        ))
    );
    let plugin_status = state
        .plugin_status
        .iter()
        .flat_map(|text| text.spans.iter().map(|span| span.text.as_str()))
        .collect::<String>();
    render_status_line(
        frame,
        format!("{summary}{plugin_status}"),
        metrics.directory_summary,
        palette::role(ThemeRole::StatusBar),
    );

    let message = state
        .message
        .as_deref()
        .unwrap_or("Enter Open  Backspace Parent  Space Mark  R Refresh  Ctrl+Q Quit");
    render_status_line(
        frame,
        message,
        metrics.message_bar,
        palette::role(ThemeRole::MessageBar),
    );

    frame.render_widget(
        Paragraph::new(pad_or_truncate(
            &format!(
                "{}{}",
                state.registry.function_bar_text(),
                plugin_command_footer(&state.plugin_commands)
            ),
            metrics.function_bar.width as usize,
        ))
        .style(palette::role(ThemeRole::FunctionBar)),
        metrics.function_bar,
    );
}

fn format_entry_with_decoration(
    entry: &FileEntry,
    width: usize,
    decoration: Option<&crate::plugins::api::FileDecoration>,
) -> String {
    let prefix = decoration.map_or_else(String::new, |decoration| {
        decoration
            .text
            .spans
            .iter()
            .map(|span| span.text.as_str())
            .collect()
    });
    let reserved = decoration.map_or(0, |decoration| usize::from(decoration.reserved_cells));
    format!(
        "{}{}",
        pad_or_truncate(&prefix, reserved.min(width)),
        format_entry(entry, width.saturating_sub(reserved))
    )
}

fn render_long_view(
    frame: &mut Frame<'_>,
    state: &AppState,
    metrics: &LayoutMetrics,
    page_start: usize,
) {
    let Some(first) = metrics.columns.first() else {
        return;
    };
    let Some(last) = metrics.columns.last() else {
        return;
    };
    let area = Rect::new(
        first.x,
        first.y,
        last.x + last.width - first.x,
        first.height,
    );
    let width = area.width as usize;
    let show_attr = width >= 92;
    let show_time = width >= 72;
    let show_date = width >= 62;
    let fixed =
        10 + usize::from(show_attr) * 6 + usize::from(show_time) * 7 + usize::from(show_date) * 7;
    let name_width = width.saturating_sub(fixed).max(8);
    let mut lines = vec![Line::raw(format!(
        "{} {:>9}{}{}{}",
        pad_or_truncate("Name", name_width),
        "Size",
        if show_date { "   Date" } else { "" },
        if show_time { "   Time" } else { "" },
        if show_attr { "   Attr" } else { "" },
    ))];
    for (offset, entry) in state
        .entries
        .iter()
        .skip(page_start)
        .take(area.height.saturating_sub(1) as usize)
        .enumerate()
    {
        let modified = entry.local_modified;
        let text = format!(
            "{} {:>9}{}{}{}",
            pad_or_truncate(&entry.display_name(), name_width),
            match entry.kind {
                EntryKind::Directory => "<DIR>".to_string(),
                EntryKind::Parent => "<UP>".to_string(),
                _ => human_size(entry.size),
            },
            if show_date {
                modified
                    .map(|value| format!(" {:02}-{:02}", value.month, value.day))
                    .unwrap_or_else(|| " -----".to_string())
            } else {
                String::new()
            },
            if show_time {
                modified
                    .map(|value| format!(" {:02}:{:02}", value.hour, value.minute))
                    .unwrap_or_else(|| " --:--".to_string())
            } else {
                String::new()
            },
            if show_attr {
                format!("   {}", format_attributes(entry))
            } else {
                String::new()
            },
        );
        let index = page_start + offset;
        lines.push(Line::from(Span::styled(
            text,
            palette::entry(
                entry,
                index == state.selected,
                state.marked.contains(&entry.path),
            ),
        )));
    }
    frame.render_widget(
        Paragraph::new(lines).style(palette::role(ThemeRole::MainBackground)),
        area,
    );
}

fn render_status_line(frame: &mut Frame<'_>, text: impl AsRef<str>, area: Rect, style: Style) {
    frame.render_widget(
        Paragraph::new(pad_or_truncate(text.as_ref(), area.width as usize)).style(style),
        area,
    );
}

fn render_help(
    frame: &mut Frame<'_>,
    viewport: Rect,
    registry: &CommandRegistry,
    plugins: &[crate::app::command_registry::PluginCommandHint],
) {
    let width = viewport.width.saturating_sub(8).min(64);
    let height = viewport.height.saturating_sub(4).min(17);
    let area = centered_rect(width, height, viewport);
    frame.render_widget(Clear, area);
    let mut help = vec![Line::raw("Active commands")];
    help.extend(registry.active_help_lines().into_iter().map(Line::raw));
    help.extend(plugins.iter().map(|command| {
        let key = command
            .key
            .map_or("(no key)".to_string(), |key| key.display());
        let suffix = match &command.availability {
            crate::plugins::api::CommandAvailability::Enabled => String::new(),
            crate::plugins::api::CommandAvailability::Disabled { reason } => {
                format!(" [disabled: {reason}]")
            }
        };
        Line::raw(format!("{key:<11} {}{suffix}", command.label))
    }));
    help.push(Line::raw("Esc         Close this window"));
    frame.render_widget(
        Paragraph::new(help)
            .block(
                Block::default()
                    .title(" Mdir4 Help ")
                    .borders(Borders::ALL)
                    .border_style(palette::role(ThemeRole::DialogBorder))
                    .style(palette::role(ThemeRole::Dialog)),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn plugin_command_footer(commands: &[crate::app::command_registry::PluginCommandHint]) -> String {
    commands
        .iter()
        .filter_map(|command| match command.availability {
            crate::plugins::api::CommandAvailability::Enabled => command
                .key
                .map(|key| format!("  {}{}", key.display(), command.label)),
            crate::plugins::api::CommandAvailability::Disabled { .. } => None,
        })
        .collect()
}

fn render_quit_confirmation(frame: &mut Frame<'_>, viewport: Rect) {
    let area = centered_rect(38, 5, viewport);
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new("Quit Mdir4?\n\nEnter Confirm   Esc Cancel")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title(" Confirm Quit ")
                    .borders(Borders::ALL)
                    .border_style(palette::role(ThemeRole::DialogBorder))
                    .style(palette::role(ThemeRole::Dialog)),
            ),
        area,
    );
}

fn render_too_small(frame: &mut Frame<'_>, viewport: Rect) {
    frame.render_widget(
        Paragraph::new("Terminal too small (minimum 60x15)")
            .alignment(Alignment::Center)
            .style(palette::role(ThemeRole::Warning)),
        viewport,
    );
}

fn format_entry(entry: &FileEntry, width: usize) -> String {
    let name = entry.display_name();
    if width < 28 {
        return pad_or_truncate(&name, width);
    }
    let metadata = match entry.kind {
        EntryKind::Parent => "<UP>".to_string(),
        EntryKind::Directory => "<DIR>".to_string(),
        EntryKind::File => human_size(entry.size),
        EntryKind::Other => "<OTHER>".to_string(),
    };
    let wide = width >= 40;
    let metadata_width = 8;
    let suffix_width = if wide { 12 } else { 0 };
    let name_width = width.saturating_sub(metadata_width + suffix_width + 1);
    let compact = format!(
        "{} {:>metadata_width$}",
        pad_or_truncate(&name, name_width),
        metadata
    );
    if wide {
        format!("{compact} {}", format_modified(entry))
    } else {
        compact
    }
}

fn format_modified(entry: &FileEntry) -> String {
    entry
        .local_modified
        .map(|value| {
            format!(
                "{:02}-{:02} {:02}:{:02}",
                value.month, value.day, value.hour, value.minute
            )
        })
        .unwrap_or_else(|| "----- --:--".to_string())
}

fn format_attributes(entry: &FileEntry) -> String {
    let attributes = entry.attributes;
    [
        (attributes.read_only, 'R'),
        (attributes.hidden, 'H'),
        (attributes.system, 'S'),
        (attributes.archive, 'A'),
    ]
    .into_iter()
    .map(|(enabled, character)| if enabled { character } else { '-' })
    .collect()
}

fn human_size(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB * 1024;
    const GIB: u64 = MIB * 1024;
    if bytes >= GIB {
        format!("{:.1}G", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1}M", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.1}K", bytes as f64 / KIB as f64)
    } else {
        format!("{bytes}B")
    }
}

#[cfg(test)]
fn truncate_cells(text: &str, width: usize) -> String {
    crate::layout::text::truncate_end(text, width, "…")
}

fn centered_rect(width: u16, height: u16, outer: Rect) -> Rect {
    Rect::new(
        outer.x + outer.width.saturating_sub(width) / 2,
        outer.y + outer.height.saturating_sub(height) / 2,
        width.min(outer.width),
        height.min(outer.height),
    )
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, ffi::OsString, path::PathBuf};

    use insta::assert_snapshot;
    use ratatui::{Terminal, backend::TestBackend, layout::Position};

    use super::*;
    use crate::{app::AppState, fs::FileEntry, layout::Viewport};

    fn rendered(state: &AppState, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        let metrics = crate::layout::calculate_for_entries(
            state.viewport,
            state.layout_settings,
            state.entries.len(),
        );
        terminal
            .draw(|frame| render(frame, state, &metrics))
            .unwrap();
        let buffer = terminal.backend().buffer();
        let mut output = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                output.push_str(buffer[(x, y)].symbol());
            }
            output.push('\n');
        }
        output
    }

    fn state_with(entries: Vec<FileEntry>, width: u16, height: u16) -> AppState {
        AppState {
            current_path: PathBuf::from("/work/한글-folder"),
            entries,
            selected: 0,
            marked: HashSet::new(),
            viewport: Viewport { width, height },
            layout_settings: crate::layout::LayoutSettings::default(),
            screen: Screen::Main,
            message: None,
            free_space: None,
            should_quit: false,
            input_dialog: None,
            confirm_dialog: None,
            viewer: None,
            editor: None,
            sort_key: crate::model::directory::SortKey::Name,
            sort_direction: crate::model::directory::SortDirection::Ascending,
            show_hidden: true,
            drives: Vec::new(),
            selected_drive: 0,
            conflict: None,
            long_view: false,
            theme: crate::theme::catalog::Theme::classic(),
            mcd: None,
            qcd: Vec::new(),
            selected_qcd: 0,
            menu_category: 0,
            menu_item: 0,
            settings_cursor: 0,
            settings_preview: None,
            config_path: None,
            persisted_config: crate::config::Config::default(),
            registry: CommandRegistry::default(),
            plugin_status: Vec::new(),
            plugin_commands: Vec::new(),
            plugin_decorations: std::collections::BTreeMap::new(),
            git_status_view: None,
            git_diff: None,
        }
    }

    #[test]
    fn plugin_command_footer_excludes_disabled_hints() {
        use crate::{
            app::command_registry::PluginCommandHint,
            input::key::{KeyChord, KeyCode},
            plugins::api::CommandAvailability,
        };
        let commands = vec![
            PluginCommandHint {
                id: "plugin.fake.ok".into(),
                label: "Fake".into(),
                key: Some(KeyChord::plain(KeyCode::Function(10))),
                availability: CommandAvailability::Enabled,
            },
            PluginCommandHint {
                id: "plugin.fake.no".into(),
                label: "Hidden".into(),
                key: Some(KeyChord::plain(KeyCode::Function(11))),
                availability: CommandAvailability::Disabled {
                    reason: "Conflict".into(),
                },
            },
        ];
        assert_eq!(plugin_command_footer(&commands), "  F10Fake");
    }

    #[test]
    fn decoration_reserves_its_prefix_cells_before_formatting_the_filename() {
        let entry = entry("very-long-file-name.txt", EntryKind::File, 1);
        let decoration = crate::plugins::api::FileDecoration {
            entry_id: entry.path.display().to_string(),
            text: crate::plugins::api::StyledText {
                spans: vec![crate::plugins::api::StyledSpan {
                    text: "!!".into(),
                    role: None,
                }],
            },
            reserved_cells: 2,
            priority: 1,
        };
        let rendered = format_entry_with_decoration(&entry, 12, Some(&decoration));
        assert_eq!(&rendered[..2], "!!");
        assert_eq!(crate::layout::text::cell_width(&rendered), 12);
    }

    #[test]
    fn git_status_renders_as_a_full_screen_plugin_owned_view() {
        let mut state = state_with(Vec::new(), 80, 25);
        state.screen = Screen::GitStatus;
        state.git_status_view = Some(crate::plugins::git::status_view::GitStatusViewState {
            rows: vec![crate::plugins::git::model::GitStatusRow {
                path: crate::plugins::git::model::RepoRelativePath::new("changed.txt").unwrap(),
                status: crate::plugins::git::model::GitStatus::Modified,
                old_path: None,
            }],
            selected: 0,
            marked: std::collections::BTreeSet::new(),
        });
        let output = rendered(&state, 80, 25);
        assert!(output.contains("Git Status"));
        assert!(output.contains("M  changed.txt"));
        assert!(!output.contains("Enter Open"));
    }

    fn entry(name: &str, kind: EntryKind, size: u64) -> FileEntry {
        FileEntry::new(
            PathBuf::from("/work").join(name),
            OsString::from(name),
            kind,
            size,
        )
    }

    #[test]
    fn unicode_truncation_respects_cell_width() {
        let result = truncate_cells("한글파일.txt", 8);
        assert!(crate::layout::text::cell_width(&result) <= 8);
        assert!(result.ends_with('…'));
    }

    #[test]
    fn input_view_keeps_a_unicode_cursor_inside_the_field() {
        assert_eq!(visible_input("보고서.txt", 2, 12), ("보고서.txt".into(), 4));
        assert_eq!(visible_input("가나다라마바사", 7, 6), ("바사".into(), 4));
        assert_eq!(visible_input("a👨‍👩‍👧‍👦b", 2, 4), ("a👨‍👩‍👧‍👦b".into(), 3));
    }

    #[test]
    fn input_dialog_places_the_real_terminal_cursor_at_the_unicode_cell() {
        let mut state = state_with(Vec::new(), 80, 25);
        let mut dialog = crate::model::dialog::InputDialog::new(
            "Rename",
            "New name",
            "보고서.txt",
            crate::model::dialog::InputPurpose::Rename,
            None,
        );
        dialog.move_home();
        dialog.move_right();
        dialog.move_right();
        state.input_dialog = Some(dialog);
        state.screen = Screen::InputDialog;
        let metrics = crate::layout::calculate_for_entries(
            state.viewport,
            state.layout_settings,
            state.entries.len(),
        );
        let backend = TestBackend::new(80, 25);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| render(frame, &state, &metrics))
            .unwrap();

        assert_eq!(terminal.backend().cursor_position(), Position::new(9, 11));
    }

    #[test]
    fn viewer_uses_the_entire_viewport_without_rendering_the_main_screen() {
        let mut state = state_with(vec![entry("stale-main.txt", EntryKind::File, 42)], 80, 25);
        let viewer = crate::model::viewer::ViewerDocument::decode(b"viewer body".to_vec());
        state.viewer = Some((PathBuf::from("/work/viewer.txt"), viewer));
        state.screen = Screen::Viewer;

        let output = rendered(&state, 80, 25);
        assert!(output.starts_with("┌ View: /work/viewer.txt"));
        assert!(output.contains("viewer body"));
        assert!(output.contains("Esc Close"));
        assert!(!output.contains("stale-main.txt"));
        assert!(!output.contains("Files 1"));
    }

    #[test]
    fn startup_80x25_snapshot() {
        let mut entries = Vec::new();
        for index in 1..=28 {
            entries.push(FileEntry::new(
                PathBuf::from(format!("/work/FILE{index:03}.TXT")),
                OsString::from(format!("FILE{index:03}.TXT")),
                EntryKind::File,
                index * 1024,
            ));
        }
        let state = AppState {
            current_path: PathBuf::from("/work"),
            entries,
            selected: 2,
            marked: HashSet::from([PathBuf::from("/work/FILE002.TXT")]),
            viewport: Viewport {
                width: 80,
                height: 25,
            },
            layout_settings: crate::layout::LayoutSettings::default(),
            screen: Screen::Main,
            message: None,
            free_space: None,
            should_quit: false,
            input_dialog: None,
            confirm_dialog: None,
            viewer: None,
            editor: None,
            sort_key: crate::model::directory::SortKey::Name,
            sort_direction: crate::model::directory::SortDirection::Ascending,
            show_hidden: true,
            drives: Vec::new(),
            selected_drive: 0,
            conflict: None,
            long_view: false,
            theme: crate::theme::catalog::Theme::classic(),
            mcd: None,
            qcd: Vec::new(),
            selected_qcd: 0,
            menu_category: 0,
            menu_item: 0,
            settings_cursor: 0,
            settings_preview: None,
            config_path: None,
            persisted_config: crate::config::Config::default(),
            registry: CommandRegistry::default(),
            plugin_status: Vec::new(),
            plugin_commands: Vec::new(),
            plugin_decorations: std::collections::BTreeMap::new(),
            git_status_view: None,
            git_diff: None,
        };
        assert_snapshot!(rendered(&state, 80, 25));
    }

    #[test]
    fn column_separator_uses_box_drawing_border_cells() {
        let entries = (0..21)
            .map(|index| entry(&format!("{index}.txt"), EntryKind::File, 1))
            .collect();
        let state = state_with(entries, 80, 25);
        let metrics = crate::layout::calculate_for_entries(
            state.viewport,
            state.layout_settings,
            state.entries.len(),
        );
        assert_eq!(metrics.columns.len(), 2);
        let separator_x = metrics.columns[0].x + metrics.columns[0].width - 1;
        let backend = TestBackend::new(80, 25);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| render(frame, &state, &metrics))
            .unwrap();
        let buffer = terminal.backend().buffer();
        for y in metrics.list.y..metrics.list.y + metrics.list.height {
            assert_eq!(buffer[(separator_x, y)].symbol(), "│");
        }
    }

    #[test]
    fn m3_overlays_and_long_view_render_english_copy_and_unicode_paths() {
        let mut state = state_with(vec![entry("한글.txt", EntryKind::File, 42)], 100, 30);
        state.long_view = true;
        let long = rendered(&state, 100, 30);
        assert!(long.contains("Name"));
        assert_eq!(state.entries[0].display_name(), "한글.txt");

        let root = {
            let mut tree = crate::mcd::tree::DirectoryTree::default();
            let root = tree.add_root(PathBuf::from("/"));
            tree.set_children(root, vec![PathBuf::from("/한글")]);
            tree.expand();
            state.mcd = Some(tree);
            root
        };
        assert!(root.0 > 0);
        state.screen = Screen::Mcd;
        assert!(rendered(&state, 100, 30).contains("Mdir4 Change Directory"));

        state.qcd.push(crate::config::schema::QcdEntry {
            label: "Work 한글".to_string(),
            path: PathBuf::from("/한글"),
            position: 0,
        });
        state.screen = Screen::Qcd;
        assert!(rendered(&state, 100, 30).contains("QCD Favorites"));

        state.screen = Screen::Menu;
        assert!(rendered(&state, 100, 30).contains("Mdir4 Menu"));
        crate::app::reduce(&mut state, crate::app::Action::ShowSettings);
        let settings = rendered(&state, 100, 30);
        assert!(settings.contains("Settings Preview"));
        assert!(settings.contains("Keymap"));
    }

    #[test]
    fn mcd_clears_main_screen_and_keeps_scrolled_selection_visible() {
        let mut state = state_with(vec![entry("stale-main.txt", EntryKind::File, 42)], 80, 25);
        let mut tree = crate::mcd::tree::DirectoryTree::default();
        let root = tree.add_root(PathBuf::from("/"));
        tree.set_children(
            root,
            (0..30)
                .map(|index| PathBuf::from(format!("/item-{index:02}")))
                .collect(),
        );
        tree.expand();
        tree.page_move(4, 6);
        let selected_name = tree
            .selected_node()
            .unwrap()
            .path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        state.mcd = Some(tree);
        state.screen = Screen::Mcd;

        let output = rendered(&state, 80, 25);
        assert!(output.contains("Mdir4 Change Directory"));
        assert!(output.contains(&format!("> ├─ {selected_name}")));
        assert!(output.contains("PgUp/PgDn"));
        assert!(!output.contains("stale-main.txt"));
        assert!(!output.contains("Files 1"));
    }

    #[test]
    fn mcd_hides_partial_ancestor_tree_until_loading_finishes() {
        let mut state = state_with(Vec::new(), 80, 25);
        state.current_path = PathBuf::from("/Users/me/project");
        let mut tree = crate::mcd::tree::DirectoryTree::default();
        let root = tree.add_root(PathBuf::from("/"));
        tree.reveal_path(&state.current_path);
        tree.set_loading(root);
        state.mcd = Some(tree);
        state.screen = Screen::Mcd;

        let output = rendered(&state, 80, 25);
        assert!(output.contains("Loading current directory tree"));
        assert!(!output.contains("Users"));
        assert!(!output.contains("project"));
    }

    #[test]
    fn required_screen_snapshots() {
        let empty = state_with(Vec::new(), 80, 25);
        assert_snapshot!("empty", rendered(&empty, 80, 25));

        let items = vec![
            entry("docs", EntryKind::Directory, 0),
            entry("README.md", EntryKind::File, 2048),
            entry("한글-문서.txt", EntryKind::File, 42),
        ];
        let mut basic = state_with(items, 80, 25);
        assert_snapshot!("basic", rendered(&basic, 80, 25));
        assert_snapshot!("unicode", rendered(&basic, 80, 25));

        basic.selected = 1;
        basic.marked.insert(PathBuf::from("/work/README.md"));
        assert_snapshot!("marked", rendered(&basic, 80, 25));

        basic.screen = Screen::Help;
        assert_snapshot!("help", rendered(&basic, 80, 25));

        basic.screen = Screen::QuitConfirm;
        assert_snapshot!("quit-confirm", rendered(&basic, 80, 25));

        basic.screen = Screen::Main;
        assert_snapshot!("viewport-80x25", rendered(&basic, 80, 25));

        let wide = state_with(
            vec![entry("wide-density.txt", EntryKind::File, 8192)],
            120,
            40,
        );
        assert_snapshot!("viewport-120x40", rendered(&wide, 120, 40));

        let small = state_with(Vec::new(), 59, 14);
        assert_snapshot!("too-small", rendered(&small, 59, 14));
    }
}
