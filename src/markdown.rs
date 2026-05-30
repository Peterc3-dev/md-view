use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use crate::syntax;

/// A rendered line with its associated metadata
#[derive(Clone, Debug)]
pub struct RenderedLine {
    pub line: Line<'static>,
    pub line_type: LineType,
    /// For header lines, the original header text (for TOC / search)
    pub plain_text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LineType {
    Normal,
    Header(u8),
    CodeBlock,
    BlockQuote,
    ListItem(u8),
    HorizontalRule,
    TableRow,
    TableSeparator,
    Empty,
}

/// Theme colors — phosphor green palette
pub struct Theme;

impl Theme {
    pub fn h1() -> Style {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    }
    pub fn h2() -> Style {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    }
    pub fn h3() -> Style {
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    }
    pub fn h4() -> Style {
        Style::default().fg(Color::DarkGray)
    }
    pub fn bold() -> Style {
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::White)
    }
    pub fn italic() -> Style {
        Style::default()
            .add_modifier(Modifier::ITALIC | Modifier::UNDERLINED)
            .fg(Color::LightGreen)
    }
    pub fn code_inline() -> Style {
        Style::default().fg(Color::Cyan)
    }
    pub fn code_block_border() -> Style {
        Style::default().fg(Color::DarkGray)
    }
    pub fn link_text() -> Style {
        Style::default()
            .fg(Color::LightCyan)
            .add_modifier(Modifier::UNDERLINED)
    }
    pub fn link_url() -> Style {
        Style::default().fg(Color::DarkGray)
    }
    pub fn blockquote_bar() -> Style {
        Style::default().fg(Color::Green)
    }
    pub fn blockquote_text() -> Style {
        Style::default().fg(Color::DarkGray)
    }
    pub fn list_bullet() -> Style {
        Style::default().fg(Color::Green)
    }
    pub fn strikethrough() -> Style {
        Style::default()
            .add_modifier(Modifier::CROSSED_OUT)
            .fg(Color::DarkGray)
    }
    pub fn hr() -> Style {
        Style::default().fg(Color::DarkGray)
    }
    pub fn table_border() -> Style {
        Style::default().fg(Color::DarkGray)
    }
    pub fn task_done() -> Style {
        Style::default().fg(Color::Green)
    }
    pub fn task_pending() -> Style {
        Style::default().fg(Color::Yellow)
    }
    pub fn normal() -> Style {
        Style::default().fg(Color::White)
    }
}

pub fn parse_markdown(source: &str, width: usize) -> Vec<RenderedLine> {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(source, opts);
    let events: Vec<Event<'_>> = parser.collect();

    let mut renderer = MdRenderer::new(width);
    renderer.render(&events);
    renderer.output
}

struct MdRenderer {
    output: Vec<RenderedLine>,
    width: usize,
    // State stacks
    style_stack: Vec<Style>,
    current_spans: Vec<Span<'static>>,
    // Current context
    in_heading: Option<u8>,
    in_code_block: bool,
    code_block_lang: Option<String>,
    code_block_content: String,
    in_blockquote: bool,
    list_depth: u8,
    list_stack: Vec<ListKind>,
    in_link: bool,
    link_url: String,
    in_table: bool,
    table_rows: Vec<Vec<String>>,
    table_current_row: Vec<String>,
    table_alignments: Vec<pulldown_cmark::Alignment>,
    in_strikethrough: bool,
    in_table_head: bool,
}

#[derive(Clone)]
enum ListKind {
    Unordered,
    Ordered(u64),
}

impl MdRenderer {
    fn new(width: usize) -> Self {
        Self {
            output: Vec::new(),
            width,
            style_stack: vec![Theme::normal()],
            current_spans: Vec::new(),
            in_heading: None,
            in_code_block: false,
            code_block_lang: None,
            code_block_content: String::new(),
            in_blockquote: false,
            list_depth: 0,
            list_stack: Vec::new(),
            in_link: false,
            link_url: String::new(),
            in_table: false,
            table_rows: Vec::new(),
            table_current_row: Vec::new(),
            table_alignments: Vec::new(),
            in_strikethrough: false,
            in_table_head: false,
        }
    }

    fn current_style(&self) -> Style {
        self.style_stack.last().copied().unwrap_or(Theme::normal())
    }

    fn push_style(&mut self, style: Style) {
        self.style_stack.push(style);
    }

    fn pop_style(&mut self) {
        if self.style_stack.len() > 1 {
            self.style_stack.pop();
        }
    }

    fn emit_line(&mut self, line_type: LineType) {
        let spans = std::mem::take(&mut self.current_spans);
        let plain: String = spans.iter().map(|s| s.content.as_ref()).collect();
        self.output.push(RenderedLine {
            line: Line::from(spans),
            line_type,
            plain_text: plain,
        });
    }

    fn emit_empty(&mut self) {
        self.output.push(RenderedLine {
            line: Line::from(""),
            line_type: LineType::Empty,
            plain_text: String::new(),
        });
    }

    fn render(&mut self, events: &[Event<'_>]) {
        for event in events {
            match event {
                Event::Start(tag) => self.start_tag(tag),
                Event::End(tag) => self.end_tag(tag),
                Event::Text(text) => self.text(text),
                Event::Code(code) => self.inline_code(code),
                Event::SoftBreak => self.soft_break(),
                Event::HardBreak => self.hard_break(),
                Event::Rule => self.horizontal_rule(),
                Event::TaskListMarker(checked) => self.task_list_marker(*checked),
                _ => {}
            }
        }
    }

    fn start_tag(&mut self, tag: &Tag<'_>) {
        match tag {
            Tag::Heading { level, .. } => {
                let lvl = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                self.in_heading = Some(lvl);
                let style = match lvl {
                    1 => Theme::h1(),
                    2 => Theme::h2(),
                    3 => Theme::h3(),
                    _ => Theme::h4(),
                };
                self.push_style(style);
                // Add heading prefix
                let prefix = match lvl {
                    1 => "█ ",
                    2 => "▓ ",
                    3 => "▒ ",
                    _ => "░ ",
                };
                self.current_spans
                    .push(Span::styled(prefix.to_string(), style));
            }
            Tag::Paragraph => {
                // Nothing special at start
            }
            Tag::BlockQuote(_) => {
                self.in_blockquote = true;
            }
            Tag::CodeBlock(kind) => {
                self.in_code_block = true;
                self.code_block_content.clear();
                self.code_block_lang = match kind {
                    CodeBlockKind::Fenced(lang) => {
                        let l = lang.to_string();
                        if l.is_empty() {
                            None
                        } else {
                            Some(l)
                        }
                    }
                    CodeBlockKind::Indented => None,
                };
            }
            Tag::List(start) => {
                match start {
                    Some(n) => self.list_stack.push(ListKind::Ordered(*n)),
                    None => self.list_stack.push(ListKind::Unordered),
                }
                self.list_depth = self.list_stack.len() as u8;
            }
            Tag::Item => {
                let indent = "  ".repeat(self.list_depth.saturating_sub(1) as usize);
                let bullet = match self.list_stack.last() {
                    Some(ListKind::Unordered) => match self.list_depth {
                        1 => "•",
                        2 => "◦",
                        _ => "▪",
                    },
                    Some(ListKind::Ordered(n)) => {
                        let marker = format!("{}.", n);
                        // Update the counter
                        if let Some(ListKind::Ordered(ref mut num)) = self.list_stack.last_mut() {
                            *num += 1;
                        }
                        self.current_spans.push(Span::styled(
                            format!("{}{} ", indent, marker),
                            Theme::list_bullet(),
                        ));
                        return;
                    }
                    None => "•",
                };
                self.current_spans.push(Span::styled(
                    format!("{}{} ", indent, bullet),
                    Theme::list_bullet(),
                ));
            }
            Tag::Emphasis => {
                self.push_style(Theme::italic());
            }
            Tag::Strong => {
                self.push_style(Theme::bold());
            }
            Tag::Strikethrough => {
                self.in_strikethrough = true;
                self.push_style(Theme::strikethrough());
            }
            Tag::Link { dest_url, .. } => {
                self.in_link = true;
                self.link_url = dest_url.to_string();
                self.push_style(Theme::link_text());
            }
            Tag::Table(alignments) => {
                self.in_table = true;
                self.table_rows.clear();
                self.table_alignments = alignments.clone();
            }
            Tag::TableHead => {
                self.in_table_head = true;
                self.table_current_row.clear();
            }
            Tag::TableRow => {
                self.table_current_row.clear();
            }
            Tag::TableCell => {
                self.table_current_row.push(String::new());
            }
            _ => {}
        }
    }

    fn end_tag(&mut self, tag: &TagEnd) {
        match tag {
            TagEnd::Heading(_level) => {
                let lvl = self.in_heading.unwrap_or(1);
                self.emit_line(LineType::Header(lvl));
                self.emit_empty();
                self.pop_style();
                self.in_heading = None;
            }
            TagEnd::Paragraph => {
                if self.in_blockquote {
                    // Wrap blockquote lines
                    let spans = std::mem::take(&mut self.current_spans);
                    let plain: String = spans.iter().map(|s| s.content.as_ref()).collect();
                    let wrapped = word_wrap(&plain, self.width.saturating_sub(4));
                    for wline in wrapped {
                        let mut line_spans = vec![Span::styled(" ▌ ", Theme::blockquote_bar())];
                        line_spans.push(Span::styled(wline.clone(), Theme::blockquote_text()));
                        self.output.push(RenderedLine {
                            line: Line::from(line_spans),
                            line_type: LineType::BlockQuote,
                            plain_text: wline,
                        });
                    }
                } else {
                    // Wrap normal paragraph
                    let spans = std::mem::take(&mut self.current_spans);
                    let plain: String = spans.iter().map(|s| s.content.as_ref()).collect();
                    let style = if spans.len() == 1 {
                        spans[0].style
                    } else if !spans.is_empty() {
                        // Find the primary non-default style
                        spans
                            .iter()
                            .find(|s| s.style != Style::default())
                            .map(|s| s.style)
                            .unwrap_or(Theme::normal())
                    } else {
                        Theme::normal()
                    };

                    let wrapped = word_wrap(&plain, self.width.saturating_sub(2));
                    if wrapped.is_empty() {
                        self.emit_empty();
                    } else {
                        // For mixed-style paragraphs, we need to re-distribute spans across wrapped lines
                        // For simplicity, re-style with the dominant style
                        // For complex paragraphs, we preserve the original spans on first line
                        // and use plain text for wrapping
                        if wrapped.len() == 1 {
                            // Single line — push original spans back
                            self.current_spans = spans;
                            self.emit_line(LineType::Normal);
                        } else {
                            // Multiple lines — use word-wrapped plain text with style
                            for wline in &wrapped {
                                self.current_spans.push(Span::styled(wline.clone(), style));
                                self.emit_line(LineType::Normal);
                            }
                        }
                    }
                }
                self.emit_empty();
            }
            TagEnd::BlockQuote(_) => {
                self.in_blockquote = false;
                self.emit_empty();
            }
            TagEnd::CodeBlock => {
                self.render_code_block();
                self.in_code_block = false;
                self.code_block_lang = None;
            }
            TagEnd::List(_) => {
                self.list_stack.pop();
                self.list_depth = self.list_stack.len() as u8;
                if self.list_depth == 0 {
                    self.emit_empty();
                }
            }
            TagEnd::Item => {
                self.emit_line(LineType::ListItem(self.list_depth));
            }
            TagEnd::Emphasis => {
                self.pop_style();
            }
            TagEnd::Strong => {
                self.pop_style();
            }
            TagEnd::Strikethrough => {
                self.in_strikethrough = false;
                self.pop_style();
            }
            TagEnd::Link => {
                self.pop_style();
                if !self.link_url.is_empty() && !self.link_url.starts_with('#') {
                    self.current_spans.push(Span::styled(
                        format!(" ({})", self.link_url),
                        Theme::link_url(),
                    ));
                }
                self.in_link = false;
                self.link_url.clear();
            }
            TagEnd::Table => {
                self.render_table();
                self.in_table = false;
                self.emit_empty();
            }
            TagEnd::TableHead => {
                self.in_table_head = false;
                self.table_rows.push(self.table_current_row.clone());
            }
            TagEnd::TableRow => {
                self.table_rows.push(self.table_current_row.clone());
            }
            TagEnd::TableCell => {}
            _ => {}
        }
    }

    fn text(&mut self, text: &pulldown_cmark::CowStr<'_>) {
        if self.in_code_block {
            self.code_block_content.push_str(text);
            return;
        }

        if self.in_table {
            // Accumulate cell text
            if let Some(last) = self.table_current_row.last_mut() {
                last.push_str(text);
            } else {
                self.table_current_row.push(text.to_string());
            }
            return;
        }

        let style = self.current_style();
        self.current_spans
            .push(Span::styled(text.to_string(), style));
    }

    fn inline_code(&mut self, code: &pulldown_cmark::CowStr<'_>) {
        self.current_spans
            .push(Span::styled(format!("`{}`", code), Theme::code_inline()));
    }

    fn soft_break(&mut self) {
        self.current_spans.push(Span::raw(" "));
    }

    fn hard_break(&mut self) {
        self.emit_line(LineType::Normal);
    }

    fn horizontal_rule(&mut self) {
        let rule = "─".repeat(self.width.saturating_sub(2));
        self.current_spans
            .push(Span::styled(rule.clone(), Theme::hr()));
        self.emit_line(LineType::HorizontalRule);
        self.emit_empty();
    }

    fn task_list_marker(&mut self, checked: bool) {
        if checked {
            self.current_spans
                .push(Span::styled("[✓] ", Theme::task_done()));
        } else {
            self.current_spans
                .push(Span::styled("[ ] ", Theme::task_pending()));
        }
    }

    fn render_code_block(&mut self) {
        let content = std::mem::take(&mut self.code_block_content);
        let lang = self.code_block_lang.clone();
        let block_width = self.width.saturating_sub(4);

        // Top border
        let lang_label = lang.as_deref().unwrap_or("");
        let top_left = if lang_label.is_empty() {
            format!("╭{}╮", "─".repeat(block_width))
        } else {
            let label = format!(" {} ", lang_label);
            let remaining = block_width.saturating_sub(label.len());
            format!("╭{}{}╮", label, "─".repeat(remaining))
        };
        self.current_spans
            .push(Span::styled(top_left, Theme::code_block_border()));
        self.emit_line(LineType::CodeBlock);

        // Content lines with syntax highlighting
        let lines: Vec<&str> = content.lines().collect();
        for line in &lines {
            let mut line_spans = Vec::new();
            line_spans.push(Span::styled("│ ", Theme::code_block_border()));

            // Apply syntax highlighting if language is known
            let highlighted = syntax::highlight(line, lang.as_deref());
            line_spans.extend(highlighted);

            // Pad to fill box
            let visible_len = visible_width(line);
            let padding = block_width.saturating_sub(visible_len + 1);
            line_spans.push(Span::raw(" ".repeat(padding)));
            line_spans.push(Span::styled(" │", Theme::code_block_border()));

            let plain = line.to_string();
            self.output.push(RenderedLine {
                line: Line::from(line_spans),
                line_type: LineType::CodeBlock,
                plain_text: plain,
            });
        }

        // Bottom border
        let bottom = format!("╰{}╯", "─".repeat(block_width));
        self.current_spans
            .push(Span::styled(bottom, Theme::code_block_border()));
        self.emit_line(LineType::CodeBlock);
        self.emit_empty();
    }

    fn render_table(&mut self) {
        let rows = std::mem::take(&mut self.table_rows);
        if rows.is_empty() {
            return;
        }

        // Calculate column widths
        let num_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        let mut col_widths = vec![0usize; num_cols];
        for row in &rows {
            for (i, cell) in row.iter().enumerate() {
                if i < num_cols {
                    col_widths[i] = col_widths[i].max(cell.trim().len());
                }
            }
        }
        // Ensure minimum width
        for w in &mut col_widths {
            *w = (*w).max(3);
        }

        // Helper to build a border line
        let make_border = |left: &str, mid: &str, right: &str, fill: &str| -> String {
            let mut s = left.to_string();
            for (i, w) in col_widths.iter().enumerate() {
                s.push_str(&fill.repeat(w + 2));
                if i < col_widths.len() - 1 {
                    s.push_str(mid);
                }
            }
            s.push_str(right);
            s
        };

        // Top border
        let top = make_border("┌", "┬", "┐", "─");
        self.current_spans
            .push(Span::styled(top, Theme::table_border()));
        self.emit_line(LineType::TableSeparator);

        for (row_idx, row) in rows.iter().enumerate() {
            // Data row
            let mut spans = Vec::new();
            spans.push(Span::styled("│", Theme::table_border()));
            for (i, w) in col_widths.iter().enumerate() {
                let cell = row.get(i).map(|s| s.trim()).unwrap_or("");
                let padded = format!(" {:width$} ", cell, width = w);
                let style = if row_idx == 0 {
                    Theme::bold()
                } else {
                    Theme::normal()
                };
                spans.push(Span::styled(padded, style));
                spans.push(Span::styled("│", Theme::table_border()));
            }
            let plain: String = row.iter().map(|c| c.trim()).collect::<Vec<_>>().join(" | ");
            self.output.push(RenderedLine {
                line: Line::from(spans),
                line_type: LineType::TableRow,
                plain_text: plain,
            });

            // Separator after header or between rows
            if row_idx == 0 {
                let sep = make_border("├", "┼", "┤", "─");
                self.current_spans
                    .push(Span::styled(sep, Theme::table_border()));
                self.emit_line(LineType::TableSeparator);
            }
        }

        // Bottom border
        let bottom = make_border("└", "┴", "┘", "─");
        self.current_spans
            .push(Span::styled(bottom, Theme::table_border()));
        self.emit_line(LineType::TableSeparator);
    }
}

fn visible_width(s: &str) -> usize {
    unicode_width::UnicodeWidthStr::width(s)
}

pub(crate) fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_width = 0;

    for word in text.split_whitespace() {
        let w = visible_width(word);
        if current_width + w + if current.is_empty() { 0 } else { 1 } > max_width
            && !current.is_empty()
        {
            lines.push(current);
            current = String::new();
            current_width = 0;
        }
        if !current.is_empty() {
            current.push(' ');
            current_width += 1;
        }
        current.push_str(word);
        current_width += w;
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() && text.is_empty() {
        lines.push(String::new());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_wrap_keeps_short_text_on_one_line() {
        assert_eq!(word_wrap("hello world", 80), vec!["hello world"]);
    }

    #[test]
    fn word_wrap_breaks_at_width_boundary() {
        // Width 10 forces "hello" and "world" onto separate lines
        // ("hello world" is 11 columns, one over the limit).
        let wrapped = word_wrap("hello world", 10);
        assert_eq!(wrapped, vec!["hello", "world"]);
    }

    #[test]
    fn word_wrap_zero_width_returns_unmodified() {
        assert_eq!(word_wrap("anything here", 0), vec!["anything here"]);
    }

    #[test]
    fn word_wrap_empty_text_yields_single_empty_line() {
        assert_eq!(word_wrap("", 40), vec![String::new()]);
    }

    #[test]
    fn word_wrap_word_longer_than_width_is_not_split() {
        // A single token wider than max_width is emitted whole rather than
        // truncated or dropped.
        let wrapped = word_wrap("supercalifragilistic", 5);
        assert_eq!(wrapped, vec!["supercalifragilistic"]);
    }

    #[test]
    fn parse_markdown_marks_heading_lines() {
        let lines = parse_markdown("# Title\n\nbody text\n", 80);
        let headers: Vec<_> = lines
            .iter()
            .filter(|l| matches!(l.line_type, LineType::Header(_)))
            .collect();
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].line_type, LineType::Header(1));
        // The plain_text retains the rendered prefix; the title is present.
        assert!(headers[0].plain_text.contains("Title"));
    }

    #[test]
    fn parse_markdown_renders_nested_heading_levels() {
        let lines = parse_markdown("# H1\n\n## H2\n\n### H3\n", 80);
        let levels: Vec<u8> = lines
            .iter()
            .filter_map(|l| match l.line_type {
                LineType::Header(n) => Some(n),
                _ => None,
            })
            .collect();
        assert_eq!(levels, vec![1, 2, 3]);
    }

    #[test]
    fn parse_markdown_emits_code_block_lines() {
        let src = "```rust\nfn main() {}\n```\n";
        let lines = parse_markdown(src, 80);
        let code_lines = lines
            .iter()
            .filter(|l| l.line_type == LineType::CodeBlock)
            .count();
        // Top border + one content line + bottom border = 3.
        assert_eq!(code_lines, 3);
        assert!(lines.iter().any(|l| l.plain_text.contains("fn main")));
    }

    #[test]
    fn parse_markdown_marks_list_items() {
        let lines = parse_markdown("- one\n- two\n", 80);
        let items = lines
            .iter()
            .filter(|l| matches!(l.line_type, LineType::ListItem(_)))
            .count();
        assert_eq!(items, 2);
    }
}
