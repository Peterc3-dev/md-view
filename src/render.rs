use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};
use ratatui::Frame;

use crate::app::{App, Mode};

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    // Update viewport dimensions
    let new_width = area.width as usize;
    let new_height = area.height.saturating_sub(2) as usize; // Reserve for status bar

    if new_width != app.viewport_width {
        app.viewport_width = new_width;
        app.reparse();
    }
    app.viewport_height = new_height;

    // Main layout: content + status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1), // status bar
        ])
        .split(area);

    let content_area = chunks[0];
    let status_area = chunks[1];

    if app.toc_visible {
        // Split horizontally: TOC sidebar | content
        let h_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(content_area);

        draw_toc(f, app, h_chunks[0]);
        draw_content(f, app, h_chunks[1]);
    } else {
        draw_content(f, app, content_area);
    }

    draw_status_bar(f, app, status_area);
}

fn draw_content(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::NONE);

    let inner = block.inner(area);

    if app.show_raw {
        draw_raw_content(f, app, inner);
    } else {
        draw_rendered_content(f, app, inner);
    }

    // Scrollbar
    let total = app.total_lines();
    let viewport = inner.height as usize;
    if total > viewport {
        let mut scrollbar_state = ScrollbarState::new(total)
            .position(app.scroll_offset)
            .viewport_content_length(viewport);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .style(Style::default().fg(Color::DarkGray));
        f.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
    }
}

fn draw_rendered_content(f: &mut Frame, app: &App, area: Rect) {
    let height = area.height as usize;
    let end = (app.scroll_offset + height).min(app.rendered_lines.len());
    let start = app.scroll_offset.min(end);

    let visible: Vec<Line<'_>> = app.rendered_lines[start..end]
        .iter()
        .enumerate()
        .map(|(i, rl)| {
            let abs_line = start + i;
            // Highlight search matches if present
            if !app.search_matches.is_empty() {
                let has_match = app.search_matches.iter().any(|m| m.line_index == abs_line);
                if has_match {
                    let mut spans: Vec<Span<'_>> = rl.line.spans.to_vec();
                    spans.push(Span::styled(" ◄", Style::default().fg(Color::Yellow)));
                    return Line::from(spans);
                }
            }
            rl.line.clone()
        })
        .collect();

    let paragraph = Paragraph::new(visible);
    f.render_widget(paragraph, area);
}

fn draw_raw_content(f: &mut Frame, app: &App, area: Rect) {
    let height = area.height as usize;
    let end = (app.scroll_offset + height).min(app.raw_lines.len());
    let start = app.scroll_offset.min(end);

    let visible: Vec<Line<'_>> = app.raw_lines[start..end]
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let line_num = start + i + 1;
            Line::from(vec![
                Span::styled(
                    format!("{:>4} │ ", line_num),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(line.clone(), Style::default().fg(Color::Rgb(0, 255, 200))),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(visible);
    f.render_widget(paragraph, area);
}

fn draw_toc(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            " Table of Contents ",
            Style::default()
                .fg(Color::Rgb(0, 255, 200))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);

    let items: Vec<ListItem> = app
        .toc_entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let indent = "  ".repeat(entry.level.saturating_sub(1) as usize);
            let prefix = match entry.level {
                1 => "█ ",
                2 => "▓ ",
                3 => "▒ ",
                _ => "░ ",
            };
            let style = if i == app.toc_cursor {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(0, 255, 200))
                    .add_modifier(Modifier::BOLD)
            } else {
                match entry.level {
                    1 => Style::default()
                        .fg(Color::Rgb(0, 255, 200))
                        .add_modifier(Modifier::BOLD),
                    2 => Style::default().fg(Color::Rgb(0, 255, 200)),
                    3 => Style::default().fg(Color::DarkGray),
                    _ => Style::default().fg(Color::DarkGray),
                }
            };

            ListItem::new(Line::from(Span::styled(
                format!("{}{}{}", indent, prefix, entry.text),
                style,
            )))
        })
        .collect();

    let list = List::new(items);
    f.render_widget(block, area);
    f.render_widget(list, inner);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let mode_str = match app.mode {
        Mode::Normal => {
            if app.show_raw {
                "RAW"
            } else {
                "VIEW"
            }
        }
        Mode::Search => "SEARCH",
        Mode::Toc => "TOC",
    };

    let total = app.total_lines();
    let pct = if total == 0 {
        100
    } else {
        ((app.scroll_offset as f64 / total.max(1) as f64) * 100.0) as usize
    };

    let search_info = if !app.search_matches.is_empty() {
        format!(
            " │ [{}/{}] \"{}\"",
            app.search_index + 1,
            app.search_matches.len(),
            app.search_query
        )
    } else if !app.search_query.is_empty() {
        format!(" │ no matches for \"{}\"", app.search_query)
    } else {
        String::new()
    };

    let left = if app.mode == Mode::Search {
        format!(" {} │ /{}█", mode_str, app.search_input)
    } else {
        format!(" {} │ {}{}", mode_str, app.filename, search_info)
    };

    let right = format!("{}% │ {}/{} ", pct, app.scroll_offset + 1, total);

    let bar_width = area.width as usize;
    let padding = bar_width.saturating_sub(left.len() + right.len());
    let bar_text = format!("{}{}{}", left, " ".repeat(padding), right);

    let bar = Paragraph::new(Line::from(Span::styled(
        bar_text,
        Style::default()
            .fg(Color::Black)
            .bg(Color::Rgb(0, 255, 200)),
    )));

    f.render_widget(bar, area);
}
