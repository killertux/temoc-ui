//! A document on screen: how far it is scrolled and what is focused.

use crate::render::{Link, Marks, Motion, Rendered, Spot};
use crate::run::FileResults;
use crate::session::Opened;
use crate::temoc::TestTable;
use crate::theme::Theme;
use ratatui::layout::Size;
use ratatui::text::Line;
use std::collections::HashMap;
use std::ops::Range;

/// How many columns one step sideways moves.
const SIDEWAYS_STEP: usize = 8;

/// Text being selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Selection {
    cursor: Spot,
    /// Where the selection starts. Before that there is only a cursor.
    start: Option<Spot>,
    /// Whether the keyboard is moving the cursor.
    by_keyboard: bool,
}

/// A document on screen. Going back to a page finds it as it was left.
#[derive(Debug, Clone)]
pub struct Page {
    opened: Opened,
    rendered: Rendered,
    /// The results shown on the page.
    results: Option<FileResults>,
    size: Size,
    scroll: usize,
    /// How far wide tables and code blocks are scrolled sideways.
    left: usize,
    focused_link: Option<usize>,
    focused_table: Option<usize>,
    /// The focused row, among the rows of the focused table.
    focused_row: Option<usize>,
    selection: Option<Selection>,
}

impl Page {
    /// Shows a document from the top, with its first test table focused.
    #[must_use]
    pub fn new(opened: Opened, size: Size) -> Self {
        Self {
            rendered: opened.document().render(size.width, &HashMap::new()),
            focused_table: (!opened.tables().is_empty()).then_some(0),
            opened,
            results: None,
            size,
            scroll: 0,
            left: 0,
            focused_link: None,
            focused_row: None,
            selection: None,
        }
    }

    /// The file behind this page.
    #[must_use]
    pub fn opened(&self) -> &Opened {
        &self.opened
    }

    /// The file on screen, relative to the workspace.
    #[must_use]
    pub fn file(&self) -> &str {
        self.opened.file()
    }

    /// The tables that can run, in document order.
    #[must_use]
    pub fn tables(&self) -> &[TestTable] {
        self.opened.tables()
    }

    /// Adapts to the size of the screen.
    pub fn fit(&mut self, size: Size) {
        let width_changed = size.width != self.size.width;
        self.size = size;
        if width_changed {
            self.lay_out();
        }
        self.stay_on_the_page();
    }

    /// Shows these results on the page, if they are not there yet.
    pub fn show_results(&mut self, results: Option<&FileResults>) {
        if self.results.as_ref() != results {
            self.results = results.cloned();
            self.lay_out();
            self.stay_on_the_page();
        }
    }

    fn lay_out(&mut self) {
        let results = self.results.as_ref().map_or_else(HashMap::new, |results| {
            self.tables()
                .iter()
                .filter_map(|table| Some((table.line, results.table(table)?)))
                .collect()
        });
        self.rendered = self.opened.document().render(self.size.width, &results);
        // What was selected may no longer be in the same place.
        self.selection = None;
    }

    fn stay_on_the_page(&mut self) {
        self.scroll = self.scroll.min(self.last_scroll());
        self.left = self.left.min(self.rendered.last_left());
    }

    /// The rows on screen.
    #[must_use]
    pub fn lines(&self, theme: Theme) -> Vec<Line<'static>> {
        self.rendered.lines(
            theme,
            self.scroll,
            self.screen_height(),
            self.left,
            Marks {
                link: self.focused_link(),
                header: self.focused_table_row(),
                row: self
                    .focused_row()
                    .and_then(|line| self.rendered.row_of_table_line(line)),
                selection: self.selected_spots(),
            },
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
        let headings = self.opened.document().headings();
        let heading = headings.iter().find(|heading| heading.anchor == anchor);
        if let Some(rows) = heading.and_then(|heading| self.rendered.rows_of_line(heading.line)) {
            self.scroll = rows.start.min(self.last_scroll());
        }
    }

    /// The two ends of what is marked, in order. A cursor marks one character.
    fn selected_spots(&self) -> Option<(Spot, Spot)> {
        let selection = self.selection?;
        let start = selection.start.unwrap_or(selection.cursor);
        Some((start.min(selection.cursor), start.max(selection.cursor)))
    }

    /// The selected text. A cursor alone selects nothing.
    #[must_use]
    pub fn selected_text(&self) -> Option<String> {
        self.selection?.start?;
        let (from, to) = self.selected_spots()?;
        Some(self.rendered.text_between(from, to))
    }

    /// Whether the keyboard is moving a cursor through the text.
    #[must_use]
    pub fn is_selecting(&self) -> bool {
        self.selection
            .is_some_and(|selection| selection.by_keyboard)
    }

    /// Whether the keyboard cursor has marked where the selection starts.
    #[must_use]
    pub fn has_start(&self) -> bool {
        self.selection
            .is_some_and(|selection| selection.start.is_some())
    }

    /// Puts a cursor on the first character on screen.
    pub fn start_selecting(&mut self) {
        let cursor = self.rendered.first_spot(self.scroll, self.screen_height());
        self.selection = Some(Selection {
            cursor,
            start: None,
            by_keyboard: true,
        });
        self.follow_cursor(cursor);
    }

    /// Starts the selection at the cursor.
    pub fn mark_start(&mut self) {
        if let Some(selection) = &mut self.selection {
            selection.start = Some(selection.cursor);
        }
    }

    /// Moves the cursor, and the view along with it.
    pub fn move_cursor(&mut self, motion: Motion) {
        if let Some(selection) = &mut self.selection {
            selection.cursor = self.rendered.moved(selection.cursor, motion);
            let cursor = selection.cursor;
            self.follow_cursor(cursor);
        }
    }

    fn follow_cursor(&mut self, cursor: Spot) {
        self.reveal(&(cursor.row..cursor.row.saturating_add(1)));
        self.left = self.rendered.left_showing_spot(cursor, self.left);
    }

    /// Selects what lies between two spots of the screen.
    pub fn drag(&mut self, from: (u16, u16), to: (u16, u16)) {
        let spot = |(column, row): (u16, u16)| {
            let last = self.screen_height().saturating_sub(1);
            let row = self.scroll.saturating_add(usize::from(row).min(last));
            self.rendered.spot_at(row, usize::from(column), self.left)
        };
        self.selection = Some(Selection {
            cursor: spot(to),
            start: Some(spot(from)),
            by_keyboard: false,
        });
    }

    /// Forgets the selection, and any cursor.
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    /// Reacts to a click on the screen. A test table there gets the focus, and
    /// so do a row of it and a link. Anything else clears the focused row. Says
    /// whether a link was clicked.
    pub fn click(&mut self, column: u16, row: u16) -> bool {
        if row >= self.size.height {
            return false;
        }
        let row = self.scroll.saturating_add(usize::from(row));
        let holds_row = |table: &TestTable| {
            let rows = self.rendered.rows_of_line(table.line);
            rows.is_some_and(|rows| rows.contains(&row))
        };
        if let Some(index) = self.tables().iter().position(holds_row) {
            self.focused_table = Some(index);
        }
        let clicked = self.rendered.table_line_at(row);
        let table = self.focused_table();
        self.focused_row = clicked.and_then(|clicked| {
            let rows = &table?.rows;
            rows.iter().position(|row| *row == clicked)
        });
        let link = self.rendered.link_at(row, usize::from(column), self.left);
        if let Some(index) = link {
            self.focus_link(index);
        }
        link.is_some()
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

    /// The focused test table.
    #[must_use]
    pub fn focused_table(&self) -> Option<&TestTable> {
        self.tables().get(self.focused_table?)
    }

    /// The row holding the header of the focused test table.
    fn focused_table_row(&self) -> Option<usize> {
        let table = self.focused_table()?;
        Some(self.rendered.rows_of_line(table.line)?.start)
    }

    /// Focuses the next test table, stopping at the last one.
    pub fn next_table(&mut self) {
        let last = self.tables().len().saturating_sub(1);
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
        if self.focused_table != Some(index) {
            self.focused_row = None;
        }
        self.focused_table = Some(index);
        if let Some(rows) = self
            .tables()
            .get(index)
            .and_then(|table| self.table_rows(table))
        {
            self.reveal(&rows);
        }
    }

    /// The line of the focused row of the focused table.
    #[must_use]
    pub fn focused_row(&self) -> Option<usize> {
        self.focused_table()?.rows.get(self.focused_row?).copied()
    }

    /// Focuses the next row of the focused table, starting at the first one and
    /// stopping at the last one.
    pub fn next_row(&mut self) {
        let last = self.last_row();
        let next = self.focused_row.map_or(0, |index| index.saturating_add(1));
        self.focus_row(last.map(|last| next.min(last)));
    }

    /// Focuses the previous row of the focused table, starting at the last one
    /// and stopping at the first one.
    pub fn previous_row(&mut self) {
        let last = self.last_row();
        let focused = self.focused_row;
        self.focus_row(last.map(|last| focused.map_or(last, |index| index.saturating_sub(1))));
    }

    /// The position of the last row of the focused table.
    fn last_row(&self) -> Option<usize> {
        self.focused_table()?.rows.len().checked_sub(1)
    }

    fn focus_row(&mut self, index: Option<usize>) {
        self.focused_row = index;
        let line = self.focused_row();
        if let Some(row) = line.and_then(|line| self.rendered.row_of_table_line(line)) {
            self.reveal(&(row..row.saturating_add(1)));
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
    use crate::document::Document;

    fn page(markdown: &str, width: u16, height: u16) -> Page {
        let document = Document::parse(markdown).unwrap();
        let opened = Opened::new("a.md".into(), document);
        Page::new(opened, Size::new(width, height))
    }

    fn shown(page: &Page) -> Vec<String> {
        let lines = page.lines(Theme::default());
        lines.iter().map(ToString::to_string).collect()
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

    const TWO_TABLES: &str = "\
[//]: # \"decisionTable A\"

| a |
| - |
| 1 |

Between.

[//]: # \"decisionTable B\"

| b |
| - |
| 2 |
";

    fn focused_table(page: &Page) -> Option<&str> {
        page.focused_table().map(|table| table.name.as_str())
    }

    #[test]
    fn clicking_a_table_focuses_it_without_moving_the_page() {
        let mut page = page(TWO_TABLES, 20, 4);
        page.scroll_down(1);
        let top = page.scroll;
        assert!(!page.click(0, 3));
        assert_eq!(focused_table(&page), Some("A"));
        page.scroll_down(4);
        let scroll = page.scroll;
        assert!(!page.click(0, 2));
        assert_eq!(focused_table(&page), Some("B"));
        assert_eq!(page.scroll, scroll);
        assert_ne!(top, scroll);
    }

    #[test]
    fn clicking_between_tables_leaves_the_focus() {
        let mut page = page(TWO_TABLES, 20, 20);
        page.next_table();
        page.click(0, 4);
        assert_eq!(focused_table(&page), Some("B"));
    }

    #[test]
    fn clicking_a_link_in_a_scrolled_page_finds_it() {
        let markdown = format!("{}\n\n[link](1.md)", numbered(5));
        let mut page = page(&markdown, 20, 3);
        page.bottom();
        assert!(page.click(1, 2));
        assert_eq!(page.focused_link().map(|l| l.text.as_str()), Some("link"));
    }

    #[test]
    fn a_click_below_the_page_hits_nothing() {
        let mut page = page("[link](1.md)", 20, 1);
        assert!(!page.click(0, 1));
        assert_eq!(page.focused_link(), None);
    }

    #[test]
    fn a_cursor_alone_selects_nothing() {
        let mut page = page("abc", 20, 5);
        page.start_selecting();
        assert!(page.is_selecting());
        assert_eq!(page.selected_text(), None);
        page.mark_start();
        assert_eq!(page.selected_text().as_deref(), Some("a"));
    }

    #[test]
    fn marking_again_moves_the_start() {
        let mut page = page("abc", 20, 5);
        page.start_selecting();
        page.mark_start();
        page.move_cursor(Motion::Right);
        page.mark_start();
        page.move_cursor(Motion::Right);
        assert_eq!(page.selected_text().as_deref(), Some("bc"));
    }

    #[test]
    fn the_view_follows_the_cursor_in_both_directions() {
        let mut page = page(&numbered(9), 20, 3);
        page.start_selecting();
        page.move_cursor(Motion::Bottom);
        assert_eq!(shown(&page).last().map(String::as_str), Some("line9"));
        page.move_cursor(Motion::Top);
        assert_eq!(page.scroll, 0);
    }

    #[test]
    fn the_view_follows_the_cursor_sideways() {
        let mut page = page(WIDE_CODE, 30, 5);
        page.start_selecting();
        page.move_cursor(Motion::RowEnd);
        assert_eq!(page.left, 10);
        page.move_cursor(Motion::RowStart);
        assert_eq!(page.left, 0);
    }

    #[test]
    fn a_drag_works_in_both_directions_and_on_a_scrolled_page() {
        let mut page = page(&numbered(5), 20, 3);
        page.scroll_down(2);
        page.drag((0, 0), (3, 2));
        assert_eq!(page.selected_text().as_deref(), Some("line2\n\nline"));
        page.drag((3, 2), (0, 0));
        assert_eq!(page.selected_text().as_deref(), Some("line2\n\nline"));
        assert!(!page.is_selecting());
    }

    #[test]
    fn a_drag_past_the_bottom_of_the_page_stops_there() {
        let mut page = page(&numbered(5), 20, 3);
        page.drag((0, 0), (0, 9));
        assert_eq!(page.selected_text().as_deref(), Some("line1\n\nl"));
    }

    #[test]
    fn a_new_layout_forgets_the_selection() {
        let mut page = page("one two", 20, 5);
        page.drag((0, 0), (2, 0));
        page.fit(Size::new(3, 5));
        assert_eq!(page.selected_text(), None);
    }

    #[test]
    fn a_taller_screen_keeps_the_selection() {
        let mut page = page("one two", 20, 5);
        page.drag((0, 0), (2, 0));
        page.fit(Size::new(20, 9));
        assert_eq!(page.selected_text().as_deref(), Some("one"));
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
