//! Which test table of a page is focused, and which of its rows.

use crate::temoc::TestTable;

/// The focused table and row of a page. Every method takes the tables of the
/// page, in document order.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Focus {
    table: Option<usize>,
    /// The focused row, among the rows of the focused table.
    row: Option<usize>,
}

impl Focus {
    /// The focus of a page that just opened: its first table, if it has any.
    #[must_use]
    pub fn first_of(tables: &[TestTable]) -> Self {
        Self {
            table: (!tables.is_empty()).then_some(0),
            row: None,
        }
    }

    /// The same focus once the page has changed from `before` to `after`: the
    /// table of the same name, and the row in the same place of it. Without
    /// that table, it is as when the page opens.
    #[must_use]
    pub fn carried(&self, before: &[TestTable], after: &[TestTable]) -> Self {
        let (Some(index), Some(focused)) = (self.table, self.table(before)) else {
            return Self::first_of(after);
        };
        // Tables of the same name are told apart by their order.
        let alike = before.iter().take(index);
        let earlier = alike.filter(|table| table.name == focused.name).count();
        let mut named = after
            .iter()
            .enumerate()
            .filter(|(_, table)| table.name == focused.name);
        let Some((index, table)) = named.nth(earlier) else {
            return Self::first_of(after);
        };
        Self {
            table: Some(index),
            row: self.row.filter(|row| *row < table.rows.len()),
        }
    }

    /// The focused table.
    #[must_use]
    pub fn table<'tables>(&self, tables: &'tables [TestTable]) -> Option<&'tables TestTable> {
        tables.get(self.table?)
    }

    /// The position of the focused table.
    #[must_use]
    pub fn table_index(&self) -> Option<usize> {
        self.table
    }

    /// The line of the focused row of the focused table.
    #[must_use]
    pub fn row(&self, tables: &[TestTable]) -> Option<usize> {
        self.table(tables)?.rows.get(self.row?).copied()
    }

    /// Focuses a table. Moving to another one leaves no row focused.
    pub fn focus_table(&mut self, index: usize) {
        if self.table != Some(index) {
            self.row = None;
        }
        self.table = Some(index);
    }

    /// Focuses the row of the focused table on this line. Any other line leaves
    /// no row focused.
    pub fn focus_line(&mut self, tables: &[TestTable], line: Option<usize>) {
        let table = self.table(tables);
        self.row = line.and_then(|line| {
            let rows = &table?.rows;
            rows.iter().position(|row| *row == line)
        });
    }

    /// Focuses the next table, stopping at the last one.
    pub fn next_table(&mut self, tables: &[TestTable]) {
        let last = tables.len().saturating_sub(1);
        if let Some(index) = self.table {
            self.focus_table(index.saturating_add(1).min(last));
        }
    }

    /// Focuses the previous table, stopping at the first one.
    pub fn previous_table(&mut self) {
        if let Some(index) = self.table {
            self.focus_table(index.saturating_sub(1));
        }
    }

    /// Focuses the next row of the focused table, starting at the first one and
    /// stopping at the last one.
    pub fn next_row(&mut self, tables: &[TestTable]) {
        let last = self.last_row(tables);
        let next = self.row.map_or(0, |index| index.saturating_add(1));
        self.row = last.map(|last| next.min(last));
    }

    /// Focuses the previous row of the focused table, starting at the last one
    /// and stopping at the first one.
    pub fn previous_row(&mut self, tables: &[TestTable]) {
        let last = self.last_row(tables);
        let focused = self.row;
        self.row = last.map(|last| focused.map_or(last, |index| index.saturating_sub(1)));
    }

    /// The position of the last row of the focused table.
    fn last_row(&self, tables: &[TestTable]) -> Option<usize> {
        self.table(tables)?.rows.len().checked_sub(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(line: usize, rows: &[usize]) -> TestTable {
        TestTable {
            name: format!("Table {line}"),
            directive_line: line,
            line,
            heading_line: None,
            rows: rows.to_vec(),
        }
    }

    fn tables() -> Vec<TestTable> {
        vec![table(5, &[7, 8]), table(12, &[14])]
    }

    #[test]
    fn a_page_without_tables_has_nothing_to_focus() {
        let mut focus = Focus::first_of(&[]);
        focus.next_table(&[]);
        focus.next_row(&[]);
        assert_eq!(focus.table(&[]), None);
        assert_eq!(focus.row(&[]), None);
    }

    #[test]
    fn tables_stop_at_either_end() {
        let tables = tables();
        let mut focus = Focus::first_of(&tables);
        focus.previous_table();
        assert_eq!(focus.table_index(), Some(0));
        focus.next_table(&tables);
        focus.next_table(&tables);
        assert_eq!(focus.table_index(), Some(1));
    }

    #[test]
    fn rows_start_from_the_end_they_are_entered_at_and_stop_there() {
        let tables = tables();
        let mut focus = Focus::first_of(&tables);
        focus.next_row(&tables);
        assert_eq!(focus.row(&tables), Some(7));
        focus.next_row(&tables);
        focus.next_row(&tables);
        assert_eq!(focus.row(&tables), Some(8));

        let mut focus = Focus::first_of(&tables);
        focus.previous_row(&tables);
        assert_eq!(focus.row(&tables), Some(8));
    }

    #[test]
    fn moving_to_another_table_leaves_no_row_focused() {
        let tables = tables();
        let mut focus = Focus::first_of(&tables);
        focus.next_row(&tables);
        focus.focus_table(0);
        assert_eq!(focus.row(&tables), Some(7));
        focus.next_table(&tables);
        assert_eq!(focus.row(&tables), None);
    }

    #[test]
    fn focus_follows_a_table_and_row_that_moved() {
        let before = tables();
        let mut after = vec![table(7, &[9, 10]), table(14, &[16])];
        for (moved, was) in after.iter_mut().zip(&before) {
            moved.name.clone_from(&was.name);
        }
        let mut focus = Focus::first_of(&before);
        focus.next_row(&before);
        focus.next_row(&before);
        let carried = focus.carried(&before, &after);
        assert_eq!(carried.row(&after), Some(10));
    }

    #[test]
    fn focus_finds_its_table_by_name() {
        let before = tables();
        let mut after = vec![table(3, &[4]), table(12, &[14]), table(5, &[7, 8])];
        if let Some(first) = after.first_mut() {
            first.name = "New".into();
        }
        let mut focus = Focus::first_of(&before);
        focus.next_table(&before);
        focus.next_row(&before);
        let carried = focus.carried(&before, &after);
        assert_eq!(carried.table_index(), Some(1));
        assert_eq!(carried.row(&after), Some(14));
    }

    #[test]
    fn a_row_that_is_gone_leaves_its_table_focused() {
        let before = tables();
        let after = vec![table(5, &[7])];
        let mut focus = Focus::first_of(&before);
        focus.previous_row(&before);
        let carried = focus.carried(&before, &after);
        assert_eq!(carried.table_index(), Some(0));
        assert_eq!(carried.row(&after), None);
    }

    #[test]
    fn without_its_table_focus_starts_over() {
        let before = tables();
        let after = vec![table(20, &[22])];
        let mut focus = Focus::first_of(&before);
        focus.next_table(&before);
        let carried = focus.carried(&before, &after);
        assert_eq!(carried, Focus::first_of(&after));
        assert_eq!(
            Focus::first_of(&before).carried(&before, &[]).table_index(),
            None
        );
    }

    #[test]
    fn only_a_line_of_the_focused_table_focuses_a_row() {
        let tables = tables();
        let mut focus = Focus::first_of(&tables);
        focus.focus_line(&tables, Some(8));
        assert_eq!(focus.row(&tables), Some(8));
        focus.focus_line(&tables, Some(14));
        assert_eq!(focus.row(&tables), None);
    }
}
