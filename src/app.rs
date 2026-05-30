use crate::markdown::{self, RenderedLine};
use crate::search::{self, SearchMatch};
use crate::toc::{self, TocEntry};

#[derive(Clone, Debug, PartialEq)]
pub enum Mode {
    Normal,
    Search,
    Toc,
}

pub struct App {
    pub raw_content: String,
    pub filename: String,
    pub rendered_lines: Vec<RenderedLine>,
    pub raw_lines: Vec<String>,
    pub scroll_offset: usize,
    pub viewport_height: usize,
    pub viewport_width: usize,
    pub mode: Mode,
    pub show_raw: bool,

    // Search
    pub search_input: String,
    pub search_query: String,
    pub search_matches: Vec<SearchMatch>,
    pub search_index: usize,

    // TOC
    pub toc_entries: Vec<TocEntry>,
    pub toc_visible: bool,
    pub toc_cursor: usize,
}

impl App {
    pub fn new(content: String, filename: String) -> Self {
        let raw_lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        // Use a default width; will be updated on first draw
        let rendered_lines = markdown::parse_markdown(&content, 120);
        let toc_entries = toc::extract_toc(&rendered_lines);

        Self {
            raw_content: content,
            filename,
            rendered_lines,
            raw_lines,
            scroll_offset: 0,
            viewport_height: 40,
            viewport_width: 120,
            mode: Mode::Normal,
            show_raw: false,
            search_input: String::new(),
            search_query: String::new(),
            search_matches: Vec::new(),
            search_index: 0,
            toc_entries,
            toc_visible: false,
            toc_cursor: 0,
        }
    }

    pub fn reload(&mut self, content: String) {
        self.raw_lines = content.lines().map(|l| l.to_string()).collect();
        self.rendered_lines = markdown::parse_markdown(&content, self.viewport_width);
        self.toc_entries = toc::extract_toc(&self.rendered_lines);
        self.raw_content = content;
        // Re-run search if active
        if !self.search_query.is_empty() {
            self.search_matches = search::find_matches(&self.rendered_lines, &self.search_query);
            self.search_index = 0;
        }
    }

    pub fn reparse(&mut self) {
        self.rendered_lines = markdown::parse_markdown(&self.raw_content, self.viewport_width);
        self.toc_entries = toc::extract_toc(&self.rendered_lines);
        if !self.search_query.is_empty() {
            self.search_matches = search::find_matches(&self.rendered_lines, &self.search_query);
        }
    }

    pub fn total_lines(&self) -> usize {
        if self.show_raw {
            self.raw_lines.len()
        } else {
            self.rendered_lines.len()
        }
    }

    pub fn scroll_down(&mut self, n: usize) {
        let max = self.total_lines().saturating_sub(self.viewport_height);
        self.scroll_offset = (self.scroll_offset + n).min(max);
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        let max = self.total_lines().saturating_sub(self.viewport_height);
        self.scroll_offset = max;
    }

    pub fn toggle_raw(&mut self) {
        self.show_raw = !self.show_raw;
        self.scroll_offset = 0;
    }

    pub fn toggle_toc(&mut self) {
        if self.mode == Mode::Toc {
            self.mode = Mode::Normal;
            self.toc_visible = false;
        } else {
            self.mode = Mode::Toc;
            self.toc_visible = true;
        }
    }

    pub fn toc_cursor_down(&mut self) {
        if !self.toc_entries.is_empty() {
            self.toc_cursor = (self.toc_cursor + 1).min(self.toc_entries.len() - 1);
        }
    }

    pub fn toc_cursor_up(&mut self) {
        self.toc_cursor = self.toc_cursor.saturating_sub(1);
    }

    pub fn jump_to_toc_entry(&mut self) {
        if let Some(entry) = self.toc_entries.get(self.toc_cursor) {
            self.scroll_offset = entry.line_index;
        }
    }

    pub fn execute_search(&mut self) {
        self.search_query = self.search_input.clone();
        self.search_matches = search::find_matches(&self.rendered_lines, &self.search_query);
        self.search_index = 0;
        if let Some(m) = self.search_matches.first() {
            self.scroll_offset = m.line_index;
        }
    }

    pub fn next_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.search_index = (self.search_index + 1) % self.search_matches.len();
        self.scroll_offset = self.search_matches[self.search_index].line_index;
    }

    pub fn prev_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        if self.search_index == 0 {
            self.search_index = self.search_matches.len() - 1;
        } else {
            self.search_index -= 1;
        }
        self.scroll_offset = self.search_matches[self.search_index].line_index;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build an app over a body with plenty of lines for scroll math.
    fn sample_app() -> App {
        let body = "# Title\n\nalpha\n\nbeta\n\ngamma\n\ndelta\n";
        let mut app = App::new(body.to_string(), "test.md".to_string());
        app.viewport_height = 4;
        app
    }

    #[test]
    fn scroll_down_clamps_to_max_offset() {
        let mut app = sample_app();
        let max = app.total_lines().saturating_sub(app.viewport_height);
        app.scroll_down(10_000);
        assert_eq!(app.scroll_offset, max);
    }

    #[test]
    fn scroll_up_saturates_at_zero() {
        let mut app = sample_app();
        app.scroll_down(2);
        app.scroll_up(10_000);
        assert_eq!(app.scroll_offset, 0);
    }

    #[test]
    fn scroll_to_top_and_bottom() {
        let mut app = sample_app();
        app.scroll_to_bottom();
        let max = app.total_lines().saturating_sub(app.viewport_height);
        assert_eq!(app.scroll_offset, max);
        app.scroll_to_top();
        assert_eq!(app.scroll_offset, 0);
    }

    #[test]
    fn toggle_raw_switches_source_and_resets_scroll() {
        let mut app = sample_app();
        app.scroll_down(2);
        assert!(!app.show_raw);
        app.toggle_raw();
        assert!(app.show_raw);
        assert_eq!(app.scroll_offset, 0);
        // total_lines now reflects raw line count.
        assert_eq!(app.total_lines(), app.raw_lines.len());
    }

    #[test]
    fn toggle_toc_flips_mode_and_visibility() {
        let mut app = sample_app();
        assert_eq!(app.mode, Mode::Normal);
        app.toggle_toc();
        assert_eq!(app.mode, Mode::Toc);
        assert!(app.toc_visible);
        app.toggle_toc();
        assert_eq!(app.mode, Mode::Normal);
        assert!(!app.toc_visible);
    }

    #[test]
    fn toc_cursor_clamps_within_bounds() {
        let mut app = sample_app();
        // Only one heading in the sample body.
        assert_eq!(app.toc_entries.len(), 1);
        app.toc_cursor_down();
        assert_eq!(app.toc_cursor, 0); // cannot move past last entry
        app.toc_cursor_up();
        assert_eq!(app.toc_cursor, 0); // cannot move above first entry
    }

    #[test]
    fn execute_search_records_query_and_jumps_to_first_match() {
        let mut app = sample_app();
        app.search_input = "gamma".to_string();
        app.execute_search();
        assert_eq!(app.search_query, "gamma");
        assert_eq!(app.search_matches.len(), 1);
        assert_eq!(app.search_index, 0);
        assert_eq!(app.scroll_offset, app.search_matches[0].line_index);
    }

    #[test]
    fn next_and_prev_match_wrap_around() {
        let mut app = sample_app();
        // "a" appears across several lines, giving multiple matches.
        app.search_input = "a".to_string();
        app.execute_search();
        let n = app.search_matches.len();
        assert!(n >= 2, "expected multiple matches, got {n}");

        assert_eq!(app.search_index, 0);
        app.prev_match(); // wraps to last
        assert_eq!(app.search_index, n - 1);
        app.next_match(); // wraps back to first
        assert_eq!(app.search_index, 0);
    }

    #[test]
    fn next_match_is_noop_without_matches() {
        let mut app = sample_app();
        app.next_match();
        app.prev_match();
        assert_eq!(app.search_index, 0);
    }
}
