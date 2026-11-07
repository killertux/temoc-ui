//! Following the tables of a file from one version to the next, so that their
//! results can follow them.

use crate::temoc::{Event, Failure, TestTable};
use std::collections::BTreeMap;

/// A test table, as far as telling it apart from another one goes.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Table {
    name: String,
    /// Its markdown, from its directive to its last row.
    text: String,
    directive_line: usize,
}

/// The test tables of one version of a file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Version {
    tables: Vec<Table>,
}

impl Version {
    /// The tables found in `markdown`, in document order.
    #[must_use]
    pub fn of(tables: &[TestTable], markdown: &str) -> Self {
        let lines: Vec<&str> = markdown.lines().collect();
        let tables = tables
            .iter()
            .map(|table| {
                // A table without rows still has its header and separator.
                let last = table.rows.last().copied();
                let last = last.unwrap_or_else(|| table.line.saturating_add(1));
                let first = table.directive_line.saturating_sub(1);
                let text = lines.get(first..last.min(lines.len())).unwrap_or_default();
                Table {
                    name: table.name.clone(),
                    text: text.join("\n"),
                    directive_line: table.directive_line,
                }
            })
            .collect();
        Self { tables }
    }

    /// Where each table of this version is in a newer one. A table is still
    /// there if one of the same name and text is, and tables that are alike
    /// are matched in order.
    #[must_use]
    pub fn moves_to(&self, newer: &Self) -> Moves {
        let mut taken = vec![false; newer.tables.len()];
        let mut moves = BTreeMap::new();
        for table in &self.tables {
            let found = newer.tables.iter().enumerate().find(|(index, other)| {
                let free = !taken.get(*index).copied().unwrap_or(true);
                free && other.name == table.name && other.text == table.text
            });
            if let Some((index, other)) = found {
                if let Some(slot) = taken.get_mut(index) {
                    *slot = true;
                }
                moves.insert(table.directive_line, other.directive_line);
            }
        }
        Moves { tables: moves }
    }
}

/// Where tables went: the line of each directive that is still there, and the
/// line it is on now. A table that is not listed is gone.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Moves {
    tables: BTreeMap<usize, usize>,
}

impl FromIterator<(usize, usize)> for Moves {
    /// Moves from the line of each directive to the line it is on now.
    fn from_iter<Pairs: IntoIterator<Item = (usize, usize)>>(pairs: Pairs) -> Self {
        Self {
            tables: pairs.into_iter().collect(),
        }
    }
}

impl Moves {
    /// Every one of these tables, where it is.
    #[must_use]
    pub fn staying(tables: &[TestTable]) -> Self {
        let lines = tables.iter().map(|table| table.directive_line);
        Self {
            tables: lines.map(|line| (line, line)).collect(),
        }
    }

    /// Where the table with its directive on this line went, if it is still there.
    #[must_use]
    pub fn table(&self, directive_line: usize) -> Option<usize> {
        self.tables.get(&directive_line).copied()
    }

    /// Where a line of the table with its directive on `directive_line` went.
    #[must_use]
    pub fn line(&self, directive_line: usize, line: usize) -> Option<usize> {
        let now = self.table(directive_line)?;
        line.checked_add(now)?.checked_sub(directive_line)
    }

    /// These moves, and then the next ones.
    #[must_use]
    pub fn then(&self, next: &Self) -> Self {
        let tables = self.tables.iter();
        let tables = tables.filter_map(|(first, now)| Some((*first, next.table(*now)?)));
        Self {
            tables: tables.collect(),
        }
    }

    /// The tables that are still there, where they are now.
    #[must_use]
    pub fn tables(&self, tables: &[TestTable]) -> Vec<TestTable> {
        tables
            .iter()
            .filter_map(|table| {
                let at = |line| self.line(table.directive_line, line);
                let rows: Option<Vec<usize>> = table.rows.iter().map(|row| at(*row)).collect();
                let heading_line = match table.heading_line {
                    Some(line) => Some(at(line)?),
                    None => None,
                };
                Some(TestTable {
                    name: table.name.clone(),
                    directive_line: self.table(table.directive_line)?,
                    line: at(table.line)?,
                    heading_line,
                    rows: rows?,
                })
            })
            .collect()
    }

    /// A failure in the table with its directive on `directive_line`, where it is now.
    #[must_use]
    pub fn failure(&self, directive_line: usize, failure: Failure) -> Option<Failure> {
        let line = self.line(directive_line, failure.line)?;
        Some(Failure { line, ..failure })
    }

    /// Something temoc said about `tables` as they were, said about them as they
    /// are now. Nothing is said about a table that is gone.
    #[must_use]
    pub fn event(&self, tables: &[TestTable], event: Event) -> Option<Event> {
        let failures = |table: usize, failures: Vec<Failure>| -> Option<Vec<Failure>> {
            let moved = failures.into_iter();
            moved.map(|failure| self.failure(table, failure)).collect()
        };
        Some(match event {
            Event::TableStarted { line } => Event::TableStarted {
                line: self.table(line)?,
            },
            Event::TableFinished { line } => Event::TableFinished {
                line: self.table(line)?,
            },
            Event::RowStarted { table, line } => Event::RowStarted {
                table: self.table(table)?,
                line: self.line(table, line)?,
            },
            Event::RowFinished {
                table,
                line,
                result,
                failures: found,
            } => Event::RowFinished {
                table: self.table(table)?,
                line: self.line(table, line)?,
                result,
                failures: failures(table, found)?,
            },
            Event::SetupFailed(failure) => {
                let owner = crate::run::owner(tables, failure.line)?;
                Event::SetupFailed(self.failure(owner.directive_line, failure)?)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Document;
    use crate::temoc::RunResult;

    const PAGE: &str = "# Page

## One

[//]: # \"decisionTable Calculator\"

| a | sum? |
| - | ---- |
| 1 | 1    |

## Two

[//]: # \"decisionTable Calculator\"

| a | sum? |
| - | ---- |
| 2 | 2    |
";

    fn tables(markdown: &str) -> Vec<TestTable> {
        Document::parse(markdown).unwrap().tests().tables
    }

    fn version(markdown: &str) -> Version {
        Version::of(&tables(markdown), markdown)
    }

    fn moves(from: &str, to: &str) -> Moves {
        version(from).moves_to(&version(to))
    }

    fn moved(pairs: &[(usize, usize)]) -> Moves {
        pairs.iter().copied().collect()
    }

    #[test]
    fn the_same_file_moves_nothing() {
        assert_eq!(moves(PAGE, PAGE), moved(&[(5, 5), (13, 13)]));
    }

    #[test]
    fn tables_follow_the_lines_above_them() {
        let longer = PAGE.replacen("# Page\n", "# Page\n\nNew words.\n", 1);
        assert_eq!(moves(PAGE, &longer), moved(&[(5, 7), (13, 15)]));
    }

    #[test]
    fn a_changed_row_loses_its_table_only() {
        let changed = PAGE.replace("| 2 | 2    |", "| 2 | 3    |");
        assert_eq!(moves(PAGE, &changed), moved(&[(5, 5)]));
    }

    #[test]
    fn a_renamed_table_is_another_table() {
        let renamed = PAGE.replace("## Two", "## Three");
        assert_eq!(moves(PAGE, &renamed), moved(&[(5, 5)]));
    }

    #[test]
    fn tables_that_are_alike_are_matched_in_order() {
        let twice = "## Same\n\n[//]: # \"decisionTable C\"\n\n| a |\n| - |\n| 1 |\n";
        let both = format!("{twice}\n{twice}");
        let one_more = format!("Intro.\n\n{both}");
        assert_eq!(moves(&both, &one_more), moved(&[(3, 5), (11, 13)]));
    }

    #[test]
    fn moves_add_up() {
        let first = moved(&[(5, 7), (13, 15)]);
        let second = moved(&[(15, 20)]);
        assert_eq!(first.then(&second), moved(&[(13, 20)]));
    }

    #[test]
    fn tables_and_their_rows_move_together() {
        let longer = PAGE.replacen("# Page\n", "# Page\n\nNew words.\n", 1);
        assert_eq!(moves(PAGE, &longer).tables(&tables(PAGE)), tables(&longer));
    }

    fn failure(line: usize) -> Failure {
        Failure {
            line,
            message: "wrong".into(),
            snoozed_until: None,
            result: RunResult::Failed,
        }
    }

    #[test]
    fn events_move_with_their_table() {
        let moves = moved(&[(5, 7)]);
        let finished = Event::RowFinished {
            table: 5,
            line: 9,
            result: RunResult::Failed,
            failures: vec![failure(9)],
        };
        assert_eq!(
            moves.event(&tables(PAGE), finished),
            Some(Event::RowFinished {
                table: 7,
                line: 11,
                result: RunResult::Failed,
                failures: vec![failure(11)],
            })
        );
    }

    #[test]
    fn events_about_a_table_that_is_gone_are_dropped() {
        let moves = moved(&[(5, 7)]);
        let started = Event::RowStarted {
            table: 13,
            line: 17,
        };
        assert_eq!(moves.event(&tables(PAGE), started), None);
    }

    #[test]
    fn a_failure_above_every_table_moves_with_the_first() {
        let moves = moved(&[(5, 7), (13, 15)]);
        let failed = Event::SetupFailed(failure(1));
        assert_eq!(
            moves.event(&tables(PAGE), failed),
            Some(Event::SetupFailed(failure(3)))
        );
    }
}
