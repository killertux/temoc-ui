//! Finding files and headings in the workspace.

use crate::document::{Document, Heading};
use crate::workspace;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::Path;

/// What search knows about a markdown file.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Entry {
    file: String,
    test_file: bool,
    headings: Vec<Heading>,
}

impl Entry {
    /// A file that cannot be read or parsed is still listed, and opening it
    /// says what is wrong.
    fn read(workspace: &Path, file: String) -> Self {
        let parsed = fs::read_to_string(workspace.join(&file))
            .map_err(|error| error.to_string())
            .and_then(|text| Document::parse(&text));
        let (test_file, headings) = match parsed {
            Ok(document) => (document.tests().is_test_file, document.headings()),
            Err(_) => (false, Vec::new()),
        };
        Self {
            file,
            test_file,
            headings,
        }
    }
}

/// A search result: a file, or a heading in a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    /// The file, relative to the workspace.
    pub file: String,
    /// The heading found.
    pub heading: Option<Heading>,
}

impl Display for Hit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.file)?;
        if let Some(heading) = &self.heading {
            write!(f, " > {}", heading.text)?;
        }
        Ok(())
    }
}

/// The search box.
#[derive(Debug, Clone)]
pub struct Search {
    entries: Vec<Entry>,
    query: String,
    only_tests: bool,
    results: Vec<Hit>,
    selected: usize,
}

impl Search {
    /// Opens an empty search box over the markdown files of a workspace.
    #[must_use]
    pub fn open(workspace: &Path, only_tests: bool) -> Self {
        let files = workspace::markdown_files(workspace);
        Self::over(
            files
                .into_iter()
                .map(|file| Entry::read(workspace, file))
                .collect(),
            only_tests,
        )
    }

    fn over(entries: Vec<Entry>, only_tests: bool) -> Self {
        let mut search = Self {
            entries,
            query: String::new(),
            only_tests,
            results: Vec::new(),
            selected: 0,
        };
        search.refresh();
        search
    }

    /// What has been typed.
    #[must_use]
    pub fn query(&self) -> &str {
        &self.query
    }

    /// The results, in order.
    #[must_use]
    pub fn results(&self) -> &[Hit] {
        &self.results
    }

    /// The position of the chosen result.
    #[must_use]
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// The chosen result.
    #[must_use]
    pub fn chosen(&self) -> Option<&Hit> {
        self.results.get(self.selected)
    }

    /// Whether only test files are shown.
    #[must_use]
    pub fn only_tests(&self) -> bool {
        self.only_tests
    }

    /// Adds a typed character.
    pub fn type_character(&mut self, character: char) {
        self.query.push(character);
        self.refresh();
    }

    /// Removes the last typed character.
    pub fn delete_character(&mut self) {
        self.query.pop();
        self.refresh();
    }

    /// Shows only test files, or everything.
    pub fn toggle_only_tests(&mut self) {
        self.only_tests = !self.only_tests;
        self.refresh();
    }

    /// Chooses the next result, stopping at the last one.
    pub fn choose_next(&mut self) {
        let last = self.results.len().saturating_sub(1);
        self.selected = self.selected.saturating_add(1).min(last);
    }

    /// Chooses the previous result, stopping at the first one.
    pub fn choose_previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    fn refresh(&mut self) {
        self.selected = 0;
        let entries = || {
            self.entries
                .iter()
                .filter(|entry| !self.only_tests || entry.test_file)
        };
        let files = entries()
            .filter(|entry| matches(&self.query, &entry.file))
            .map(|entry| Hit {
                file: entry.file.clone(),
                heading: None,
            });
        let headings = entries()
            .filter(|_| !self.query.is_empty())
            .flat_map(|entry| {
                entry
                    .headings
                    .iter()
                    .filter(|heading| matches(&self.query, &heading.text))
                    .map(|heading| Hit {
                        file: entry.file.clone(),
                        heading: Some(heading.clone()),
                    })
            });
        self.results = files.chain(headings).collect();
    }
}

/// Whether the letters of `query` appear in `text` in order, ignoring case.
fn matches(query: &str, text: &str) -> bool {
    let mut letters = text.chars().flat_map(char::to_lowercase);
    query
        .chars()
        .flat_map(char::to_lowercase)
        .all(|letter| letters.any(|other| other == letter))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn heading(text: &str) -> Heading {
        Heading {
            text: text.into(),
            anchor: text.to_lowercase(),
            line: 1,
        }
    }

    fn entry(file: &str, test_file: bool, headings: &[&str]) -> Entry {
        Entry {
            file: file.into(),
            test_file,
            headings: headings.iter().map(|text| heading(text)).collect(),
        }
    }

    fn search() -> Search {
        Search::over(
            vec![
                entry("alpha.md", false, &["Beta"]),
                entry("beta.md", true, &["Alpha", "Other"]),
            ],
            false,
        )
    }

    fn typed(search: &mut Search, text: &str) {
        text.chars().for_each(|c| search.type_character(c));
    }

    fn labels(search: &Search) -> Vec<String> {
        search.results().iter().map(ToString::to_string).collect()
    }

    #[test]
    fn letters_must_appear_in_order() {
        assert!(matches("gs", "getting-started"));
        assert!(matches("GS", "getting-started"));
        assert!(!matches("sg", "stop"));
        assert!(matches("", "anything"));
    }

    #[test]
    fn a_letter_is_only_used_once() {
        assert!(!matches("aa", "a"));
    }

    #[test]
    fn an_empty_box_lists_files_without_headings() {
        assert_eq!(labels(&search()), ["alpha.md", "beta.md"]);
    }

    #[test]
    fn all_file_matches_come_before_any_heading_match() {
        let mut search = search();
        typed(&mut search, "beta");
        assert_eq!(labels(&search), ["beta.md", "alpha.md > Beta"]);
    }

    #[test]
    fn deleting_widens_the_results_again() {
        let mut search = search();
        typed(&mut search, "alphax");
        assert_eq!(labels(&search), Vec::<String>::new());
        search.delete_character();
        assert_eq!(labels(&search), ["alpha.md", "beta.md > Alpha"]);
    }

    #[test]
    fn deleting_from_an_empty_box_does_nothing() {
        let mut search = search();
        search.delete_character();
        assert_eq!(search.query(), "");
    }

    #[test]
    fn the_choice_stops_at_both_ends() {
        let mut search = search();
        search.choose_previous();
        assert_eq!(search.selected(), 0);
        search.choose_next();
        search.choose_next();
        assert_eq!(search.selected(), 1);
    }

    #[test]
    fn typing_goes_back_to_the_top_result() {
        let mut search = search();
        search.choose_next();
        typed(&mut search, "a");
        assert_eq!(search.selected(), 0);
    }

    #[test]
    fn nothing_is_chosen_without_results() {
        let mut search = search();
        typed(&mut search, "zzz");
        search.choose_next();
        assert_eq!(search.chosen(), None);
    }

    #[test]
    fn the_test_filter_also_hides_headings() {
        let mut search = search();
        search.toggle_only_tests();
        typed(&mut search, "beta");
        assert_eq!(labels(&search), ["beta.md"]);
    }

    #[test]
    fn a_file_that_cannot_be_read_is_still_listed() {
        let entry = Entry::read(Path::new("/no/such/folder"), "a.md".into());
        assert_eq!(entry, self::entry("a.md", false, &[]));
    }
}
