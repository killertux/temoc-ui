//! A document on screen: how far it is scrolled and what is focused.

use crate::render::{Document, Link, Rendered};
use crate::temoc::TestTable;
use ratatui::layout::Size;
use ratatui::text::Line;
use std::ops::Range;

/// How many columns one step sideways moves.
const SIDEWAYS_STEP: usize = 8;

/// A document on screen. Going back to a page finds it as it was left.
#[derive(Debug, Clone)]
pub struct Page {
    file: String,
    document: Document,
    rendered: Rendered,
    tables: Vec<TestTable>,
    size: Size,
    scroll: usize,
    /// How far wide tables and code blocks are scrolled sideways.
    left: usize,
    focused_link: Option<usize>,
    focused_table: Option<usize>,
}

impl Page {
    /// Shows a document from the top, with its first test table focused.
    #[must_use]
    pub fn new(file: String, document: Document, size: Size) -> Self {
        let tables = document.tests().tables;
        Self {
            file,
            rendered: document.render(size.width),
            document,
            focused_table: (!tables.is_empty()).then_some(0),
            tables,
            size,
            scroll: 0,
            left: 0,
            focused_link: None,
        }
    }

    /// The file on screen, relative to the workspace.
    #[must_use]
    pub fn file(&self) -> &str {
        &self.file
    }

    /// Adapts to the size of the screen.
    pub fn fit(&mut self, size: Size) {
        if size.width != self.size.width {
            self.rendered = self.document.render(size.width);
        }
        self.size = size;
        self.scroll = self.scroll.min(self.last_scroll());
        self.left = self.left.min(self.rendered.last_left());
    }

    /// The rows on screen.
    #[must_use]
    pub fn lines(&self) -> Vec<Line<'static>> {
        self.rendered.lines(
            self.scroll,
            self.screen_height(),
            self.left,
            self.focused_link(),
        )
    }

    fn screen_height(&self) -> usize {
        usize::from(self.size.height)
    }

    fn last_scroll(&self) -> usize {
        self.rendered.height().saturating_sub(self.screen_height())
    }

    /// Scrolls down, stopping at the bottom.
    pub fn scroll_down(&mut self, rows: usize) {
        self.scroll = self.scroll.saturating_add(rows).min(self.last_scroll());
    }

    /// Scrolls up, stopping at the top.
    pub fn scroll_up(&mut self, rows: usize) {
        self.scroll = self.scroll.saturating_sub(rows);
    }

    /// Scrolls down one screen.
    pub fn page_down(&mut self) {
        self.scroll_down(self.screen_height());
    }

    /// Scrolls up one screen.
    pub fn page_up(&mut self) {
        self.scroll_up(self.screen_height());
    }

    /// Scrolls wide tables and code blocks right, stopping at their end.
    pub fn scroll_right(&mut self) {
        let left = self.left.saturating_add(SIDEWAYS_STEP);
        self.left = left.min(self.rendered.last_left());
    }

    /// Scrolls wide tables and code blocks left, stopping at their start.
    pub fn scroll_left(&mut self) {
        self.left = self.left.saturating_sub(SIDEWAYS_STEP);
    }

    /// Goes to the top.
    pub fn top(&mut self) {
        self.scroll = 0;
    }

    /// Goes to the bottom.
    pub fn bottom(&mut self) {
        self.scroll = self.last_scroll();
    }

    /// Scrolls just enough to show these rows. Rows that do not fit are shown
    /// from their start.
    fn reveal(&mut self, rows: &Range<usize>) {
        let height = self.screen_height();
        if rows.start < self.scroll {
            self.scroll = rows.start;
        } else if rows.end > self.scroll.saturating_add(height) {
            self.scroll = rows.start.min(rows.end.saturating_sub(height));
        }
    }

    /// Scrolls to a heading. An anchor that matches no heading changes nothing.
    pub fn go_to_anchor(&mut self, anchor: &str) {
        let headings = self.document.headings();
        let heading = headings.iter().find(|heading| heading.anchor == anchor);
        if let Some(rows) = heading.and_then(|heading| self.rendered.rows_of_line(heading.line)) {
            self.scroll = rows.start.min(self.last_scroll());
        }
    }

    /// The focused link.
    #[must_use]
    pub fn focused_link(&self) -> Option<&Link> {
        self.rendered.links().get(self.focused_link?)
    }

    /// Focuses the next link, wrapping past the last one.
    pub fn next_link(&mut self) {
        let count = self.rendered.links().len();
        let next = self.focused_link.map_or(0, |index| index.saturating_add(1));
        self.focus_link(if next < count { next } else { 0 });
    }

    /// Focuses the previous link, wrapping past the first one.
    pub fn previous_link(&mut self) {
        let last = self.rendered.links().len().saturating_sub(1);
        let previous = self.focused_link.and_then(|index| index.checked_sub(1));
        self.focus_link(previous.unwrap_or(last));
    }

    fn focus_link(&mut self, index: usize) {
        let Some(link) = self.rendered.links().get(index) else {
            return;
        };
        let rows = link.rows.clone();
        self.left = self.rendered.left_showing(link, self.left);
        self.focused_link = Some(index);
        self.reveal(&rows);
    }

    /// The name of the focused test table.
    #[must_use]
    pub fn focused_table(&self) -> Option<&str> {
        let table = self.tables.get(self.focused_table?)?;
        Some(&table.name)
    }

    /// Focuses the next test table, stopping at the last one.
    pub fn next_table(&mut self) {
        let last = self.tables.len().saturating_sub(1);
        if let Some(index) = self.focused_table {
            self.focus_table(index.saturating_add(1).min(last));
        }
    }

    /// Focuses the previous test table, stopping at the first one.
    pub fn previous_table(&mut self) {
        if let Some(index) = self.focused_table {
            self.focus_table(index.saturating_sub(1));
        }
    }

    fn focus_table(&mut self, index: usize) {
        self.focused_table = Some(index);
        if let Some(rows) = self
            .tables
            .get(index)
            .and_then(|table| self.table_rows(table))
        {
            self.reveal(&rows);
        }
    }

    /// The rows of a table, along with the heading that names it when both fit
    /// on screen.
    fn table_rows(&self, table: &TestTable) -> Option<Range<usize>> {
        let rows = self.rendered.rows_of_line(table.line)?;
        let heading = table
            .heading_line
            .and_then(|line| self.rendered.rows_of_line(line));
        let start = heading
            .map(|heading| heading.start)
            .filter(|start| rows.end.saturating_sub(*start) <= self.screen_height())
            .unwrap_or(rows.start);
        Some(start..rows.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(markdown: &str, width: u16, height: u16) -> Page {
        let document = Document::parse(markdown).unwrap();
        Page::new("a.md".into(), document, Size::new(width, height))
    }

    fn shown(page: &Page) -> Vec<String> {
        page.lines().iter().map(ToString::to_string).collect()
    }

    fn numbered(count: usize) -> String {
        let lines: Vec<String> = (1..=count).map(|n| format!("line{n}")).collect();
        lines.join("\n\n")
    }

    #[test]
    fn a_short_page_does_not_scroll() {
        let mut page = page("a", 20, 5);
        page.scroll_down(1);
        page.bottom();
        assert_eq!(shown(&page), ["a"]);
    }

    #[test]
    fn scrolling_stops_with_the_screen_full() {
        let mut page = page(&numbered(3), 20, 2);
        page.scroll_down(100);
        assert_eq!(shown(&page), ["", "line3"]);
    }

    #[test]
    fn a_taller_screen_does_not_leave_the_page_scrolled_past_its_end() {
        let mut page = page(&numbered(3), 20, 2);
        page.bottom();
        page.fit(Size::new(20, 5));
        assert_eq!(shown(&page).len(), 5);
    }

    #[test]
    fn a_narrower_screen_lays_the_page_out_again() {
        let mut page = page("one two", 20, 5);
        page.fit(Size::new(3, 5));
        assert_eq!(shown(&page), ["one", "two"]);
    }

    #[test]
    fn a_focused_link_is_scrolled_into_view_in_both_directions() {
        let markdown = format!("[first](1.md)\n\n{}\n\n[last](2.md)", numbered(5));
        let mut page = page(&markdown, 20, 3);
        page.previous_link();
        assert_eq!(shown(&page).last().map(String::as_str), Some("last"));
        page.next_link();
        assert_eq!(shown(&page).first().map(String::as_str), Some("first"));
    }

    #[test]
    fn a_link_on_screen_does_not_scroll() {
        let markdown = format!("[first](1.md)\n\n{}", numbered(5));
        let mut page = page(&markdown, 20, 3);
        page.next_link();
        assert_eq!(page.scroll, 0);
    }

    #[test]
    fn a_page_without_links_has_nothing_to_focus() {
        let mut page = page("a", 20, 5);
        page.next_link();
        page.previous_link();
        assert_eq!(page.focused_link(), None);
    }

    #[test]
    fn an_unknown_anchor_stays_at_the_top() {
        let mut page = page(&format!("{}\n\n## End", numbered(9)), 20, 3);
        page.go_to_anchor("nope");
        assert_eq!(page.scroll, 0);
        page.go_to_anchor("end");
        assert_eq!(shown(&page).last().map(String::as_str), Some("End"));
    }

    const WIDE_CODE: &str = "```\n0123456789012345678901234567890123456789\n```";

    #[test]
    fn sideways_scrolling_stops_at_both_ends() {
        let mut page = page(WIDE_CODE, 30, 5);
        page.scroll_left();
        assert_eq!(page.left, 0);
        page.scroll_right();
        assert_eq!(page.left, 8);
        page.scroll_right();
        assert_eq!(page.left, 10);
        page.scroll_left();
        assert_eq!(page.left, 2);
    }

    #[test]
    fn a_page_that_fits_does_not_scroll_sideways() {
        let mut page = page("```\nshort\n```", 30, 5);
        page.scroll_right();
        assert_eq!(page.left, 0);
    }

    #[test]
    fn a_wider_screen_takes_back_sideways_scrolling() {
        let mut page = page(WIDE_CODE, 30, 5);
        page.scroll_right();
        page.scroll_right();
        page.fit(Size::new(35, 5));
        assert_eq!(page.left, 5);
    }

    #[test]
    fn focusing_a_link_scrolls_sideways_in_both_directions() {
        let filler = "x".repeat(30);
        let markdown = format!("| [near](1.md) | {filler} | [far](2.md) |\n| - | - | - |\n");
        let mut page = page(&markdown, 20, 5);
        page.next_link();
        assert_eq!(page.left, 0);
        page.next_link();
        assert!(shown(&page)[0].contains("far"));
        page.next_link();
        assert!(shown(&page)[0].contains("near"));
    }

    #[test]
    fn a_link_on_screen_does_not_scroll_sideways() {
        let filler = "x".repeat(30);
        let markdown = format!("| {filler} | [a](1.md) | {filler} |\n| - | - | - |\n");
        let mut page = page(&markdown, 50, 5);
        page.scroll_right();
        let left = page.left;
        page.next_link();
        assert_eq!(page.left, left);
    }

    #[test]
    fn a_link_outside_a_block_leaves_blocks_scrolled() {
        let mut page = page(&format!("[a](1.md)\n\n{WIDE_CODE}"), 30, 5);
        page.scroll_right();
        page.next_link();
        assert_eq!(page.left, 8);
    }

    #[test]
    fn a_table_too_far_from_its_heading_is_shown_without_it() {
        let table = "[//]: # \"decisionTable A\"\n\n| a |\n| - |\n| 1 |\n";
        let markdown = format!("# Name\n\n{table}\n{}\n\n{table}\n", numbered(9));
        let mut page = page(&markdown, 20, 4);
        page.next_table();
        assert_eq!(shown(&page).last().map(String::as_str), Some("1"));
        assert!(!shown(&page).contains(&"Name".to_owned()));
    }
}
