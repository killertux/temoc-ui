//! Browsing the workspace as a tree of folders and files.

use crate::workspace;
use std::collections::HashSet;
use std::path::{MAIN_SEPARATOR, MAIN_SEPARATOR_STR, Path};

/// What a row of the tree holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    File,
    Folder { open: bool },
}

/// A row of the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// The file or folder, relative to the workspace.
    path: String,
    /// The last part of its path.
    pub name: String,
    /// How many folders it is inside.
    pub depth: usize,
    kind: Kind,
}

impl Row {
    /// Whether it is a folder.
    #[must_use]
    pub fn is_folder(&self) -> bool {
        matches!(self.kind, Kind::Folder { .. })
    }

    /// The row as it reads on screen, without its indentation.
    #[must_use]
    pub fn label(&self) -> String {
        match self.kind {
            Kind::File => self.name.clone(),
            Kind::Folder { open: true } => format!("▾ {}", self.name),
            Kind::Folder { open: false } => format!("▸ {}", self.name),
        }
    }
}

/// The file tree.
#[derive(Debug, Clone)]
pub struct FileTree {
    files: Vec<String>,
    open: HashSet<String>,
    rows: Vec<Row>,
    selected: usize,
    /// The first row on screen.
    top: usize,
    /// Whether the rows on screen follow the selection. The wheel lets the
    /// selection go off screen, until it next moves.
    follow: bool,
}

impl FileTree {
    /// Opens the tree of a workspace, with `document` selected and the folders
    /// above it open.
    #[must_use]
    pub fn open(workspace: &Path, document: Option<&str>) -> Self {
        Self::over(workspace::markdown_files(workspace), document)
    }

    fn over(files: Vec<String>, document: Option<&str>) -> Self {
        let open = document.map_or_else(HashSet::new, folders_above);
        let rows = rows_of(&files, &open);
        let selected = rows
            .iter()
            .position(|row| Some(row.path.as_str()) == document)
            .unwrap_or(0);
        Self {
            files,
            open,
            rows,
            selected,
            top: 0,
            follow: true,
        }
    }

    /// The rows, in order.
    #[must_use]
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// The position of the selected row.
    #[must_use]
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// The selected row.
    #[must_use]
    pub fn selection(&self) -> Option<&Row> {
        self.rows.get(self.selected)
    }

    /// The selected file, relative to the workspace.
    #[must_use]
    pub fn chosen_file(&self) -> Option<&str> {
        let row = self.selection().filter(|row| row.kind == Kind::File)?;
        Some(&row.path)
    }

    /// Selects the next row, stopping at the last one.
    pub fn choose_next(&mut self) {
        let last = self.rows.len().saturating_sub(1);
        self.selected = self.selected.saturating_add(1).min(last);
        self.follow = true;
    }

    /// Selects the previous row, stopping at the first one.
    pub fn choose_previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        self.follow = true;
    }

    /// Selects a row by how far it is below the first one on screen. Says
    /// whether there is a row there.
    pub fn select_shown(&mut self, offset: usize) -> bool {
        let index = self.top.saturating_add(offset);
        let exists = index < self.rows.len();
        if exists {
            self.selected = index;
            self.follow = true;
        }
        exists
    }

    /// Scrolls down, leaving the selection where it is.
    pub fn scroll_down(&mut self, rows: usize) {
        self.top = self.top.saturating_add(rows);
        self.follow = false;
    }

    /// Scrolls up, leaving the selection where it is.
    pub fn scroll_up(&mut self, rows: usize) {
        self.top = self.top.saturating_sub(rows);
        self.follow = false;
    }

    /// Opens the selected folder.
    pub fn open_folder(&mut self) {
        if let Some(path) = self.selected_folder() {
            self.open.insert(path);
            self.rows = rows_of(&self.files, &self.open);
        }
    }

    /// Closes the selected folder.
    pub fn close_folder(&mut self) {
        if let Some(path) = self.selected_folder() {
            self.open.remove(&path);
            self.rows = rows_of(&self.files, &self.open);
        }
    }

    /// Opens the selected folder if it is closed, and closes it if it is open.
    pub fn toggle_folder(&mut self) {
        match self.selection().map(|row| row.kind) {
            Some(Kind::Folder { open: true }) => self.close_folder(),
            Some(Kind::Folder { open: false }) => self.open_folder(),
            _ => {}
        }
    }

    fn selected_folder(&self) -> Option<String> {
        let row = self.selection()?;
        matches!(row.kind, Kind::Folder { .. }).then(|| row.path.clone())
    }

    /// The first row to show when `room` rows fit, moving as little as possible
    /// to keep the selection on screen unless the wheel took it away.
    pub fn first_shown(&mut self, room: usize) -> usize {
        if self.follow && self.selected < self.top {
            self.top = self.selected;
        } else if self.follow && self.selected >= self.top.saturating_add(room) {
            self.top = self.selected.saturating_add(1).saturating_sub(room);
        }
        self.top = self.top.min(self.rows.len().saturating_sub(room));
        self.top
    }
}

/// The folders a file is inside, from the nearest up.
fn folders_above(file: &str) -> HashSet<String> {
    Path::new(file)
        .ancestors()
        .skip(1)
        .map(|folder| folder.to_string_lossy().into_owned())
        .filter(|folder| !folder.is_empty())
        .collect()
}

fn rows_of(files: &[String], open: &HashSet<String>) -> Vec<Row> {
    let mut rows = Vec::new();
    add_folder(files, open, "", 0, &mut rows);
    rows
}

/// Adds what is directly inside a folder, then does the same for its open
/// folders. `folder` is empty for the workspace itself.
fn add_folder(
    files: &[String],
    open: &HashSet<String>,
    folder: &str,
    depth: usize,
    rows: &mut Vec<Row>,
) {
    let prefix = if folder.is_empty() {
        String::new()
    } else {
        format!("{folder}{MAIN_SEPARATOR_STR}")
    };
    let (mut folders, mut names) = (Vec::new(), Vec::new());
    for inside in files.iter().filter_map(|file| file.strip_prefix(&prefix)) {
        match inside.split_once(MAIN_SEPARATOR) {
            Some((name, _)) => folders.push(name),
            None => names.push(inside),
        }
    }
    sort_ignoring_case(&mut folders);
    folders.dedup();
    sort_ignoring_case(&mut names);
    for name in folders {
        let path = format!("{prefix}{name}");
        let is_open = open.contains(&path);
        rows.push(Row {
            name: name.to_owned(),
            depth,
            kind: Kind::Folder { open: is_open },
            path: path.clone(),
        });
        if is_open {
            add_folder(files, open, &path, depth.saturating_add(1), rows);
        }
    }
    rows.extend(names.into_iter().map(|name| Row {
        path: format!("{prefix}{name}"),
        name: name.to_owned(),
        depth,
        kind: Kind::File,
    }));
}

fn sort_ignoring_case(names: &mut [&str]) {
    names.sort_by_cached_key(|name| (name.to_lowercase(), name.to_string()));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree(files: &[&str], document: Option<&str>) -> FileTree {
        FileTree::over(files.iter().map(ToString::to_string).collect(), document)
    }

    fn labels(tree: &FileTree) -> Vec<String> {
        tree.rows().iter().map(Row::label).collect()
    }

    const NESTED: [&str; 4] = ["a/b/deep.md", "a/near.md", "c/other.md", "top.md"];

    #[test]
    fn folders_come_before_files_at_every_level() {
        let tree = tree(&NESTED, Some("a/b/deep.md"));
        assert_eq!(
            labels(&tree),
            ["▾ a", "▾ b", "deep.md", "near.md", "▸ c", "top.md"]
        );
    }

    #[test]
    fn rows_know_how_deep_they_are() {
        let tree = tree(&NESTED, Some("a/b/deep.md"));
        let depths: Vec<usize> = tree.rows().iter().map(|row| row.depth).collect();
        assert_eq!(depths, [0, 1, 2, 1, 0, 0]);
    }

    #[test]
    fn names_are_ordered_ignoring_case() {
        let tree = tree(&["B/x.md", "a/x.md", "C.md", "b.md"], None);
        assert_eq!(labels(&tree), ["▸ a", "▸ B", "b.md", "C.md"]);
    }

    #[test]
    fn a_folder_sorts_by_its_name_and_not_by_the_paths_inside_it() {
        let mut files = vec!["a/x.md".to_owned(), "a-b/x.md".to_owned()];
        files.sort_by_key(|file| file.to_lowercase());
        assert_eq!(files.first().map(String::as_str), Some("a-b/x.md"));
        assert_eq!(labels(&FileTree::over(files, None)), ["▸ a", "▸ a-b"]);
    }

    #[test]
    fn folders_differing_only_by_case_stay_apart() {
        let tree = tree(&["A/x.md", "a/y.md", "A/z.md"], None);
        assert_eq!(labels(&tree), ["▸ A", "▸ a"]);
    }

    #[test]
    fn a_folder_is_not_confused_with_one_that_starts_the_same() {
        let mut tree = tree(&["a/x.md", "ab/y.md"], None);
        tree.open_folder();
        assert_eq!(labels(&tree), ["▾ a", "x.md", "▸ ab"]);
    }

    #[test]
    fn the_document_is_selected() {
        let tree = tree(&NESTED, Some("a/near.md"));
        assert_eq!(tree.chosen_file(), Some("a/near.md"));
    }

    #[test]
    fn a_document_the_tree_does_not_list_selects_the_first_row() {
        let tree = tree(&NESTED, Some(".hidden/secret.md"));
        assert_eq!(tree.selected(), 0);
        assert_eq!(labels(&tree), ["▸ a", "▸ c", "top.md"]);
    }

    #[test]
    fn an_empty_workspace_has_nothing_to_select() {
        let mut tree = tree(&[], None);
        tree.choose_next();
        tree.open_folder();
        tree.close_folder();
        assert_eq!(tree.selection(), None);
        assert_eq!(tree.chosen_file(), None);
    }

    #[test]
    fn a_folder_is_not_a_chosen_file() {
        assert_eq!(tree(&NESTED, None).chosen_file(), None);
    }

    #[test]
    fn opening_or_closing_a_file_does_nothing() {
        let mut tree = tree(&NESTED, Some("top.md"));
        let before = labels(&tree);
        tree.open_folder();
        tree.close_folder();
        assert_eq!(labels(&tree), before);
    }

    #[test]
    fn opening_an_open_folder_leaves_it_open() {
        let mut tree = tree(&NESTED, None);
        tree.open_folder();
        tree.open_folder();
        assert_eq!(labels(&tree).first().map(String::as_str), Some("▾ a"));
    }

    #[test]
    fn closing_a_folder_keeps_the_selection_on_it() {
        let mut tree = tree(&NESTED, Some("c/other.md"));
        tree.choose_previous();
        tree.close_folder();
        assert_eq!(tree.selection().map(Row::label).as_deref(), Some("▸ c"));
    }

    #[test]
    fn a_closed_folder_remembers_the_folders_open_inside_it() {
        let mut tree = tree(&NESTED, Some("a/b/deep.md"));
        tree.choose_previous();
        tree.choose_previous();
        tree.close_folder();
        assert_eq!(labels(&tree), ["▸ a", "▸ c", "top.md"]);
        tree.open_folder();
        assert_eq!(labels(&tree).get(1).map(String::as_str), Some("▾ b"));
    }

    fn numbered(count: usize) -> FileTree {
        let files: Vec<String> = (10..10 + count).map(|n| format!("{n}.md")).collect();
        FileTree::over(files, None)
    }

    #[test]
    fn the_selection_stays_on_screen_in_both_directions() {
        let mut tree = numbered(10);
        for _ in 0..5 {
            tree.choose_next();
        }
        assert_eq!(tree.first_shown(3), 3);
        tree.choose_previous();
        assert_eq!(tree.first_shown(3), 3);
        for _ in 0..3 {
            tree.choose_previous();
        }
        assert_eq!(tree.first_shown(3), 1);
    }

    #[test]
    fn the_wheel_can_leave_the_selection_behind_until_it_moves() {
        let mut tree = numbered(10);
        tree.first_shown(3);
        tree.scroll_down(3);
        assert_eq!(tree.first_shown(3), 3);
        assert_eq!(tree.selected(), 0);
        tree.choose_next();
        assert_eq!(tree.first_shown(3), 1);
    }

    #[test]
    fn the_wheel_stops_at_both_ends() {
        let mut tree = numbered(10);
        tree.scroll_up(3);
        assert_eq!(tree.first_shown(3), 0);
        tree.scroll_down(100);
        assert_eq!(tree.first_shown(3), 7);
    }

    #[test]
    fn a_row_is_selected_by_its_place_on_screen() {
        let mut tree = numbered(10);
        tree.scroll_down(3);
        tree.first_shown(3);
        assert!(tree.select_shown(1));
        assert_eq!(tree.chosen_file(), Some("14.md"));
    }

    #[test]
    fn below_the_last_row_nothing_is_selected() {
        let mut tree = numbered(2);
        assert!(!tree.select_shown(2));
        assert_eq!(tree.selected(), 0);
    }

    #[test]
    fn toggling_opens_and_closes_only_folders() {
        let mut tree = tree(&NESTED, None);
        tree.toggle_folder();
        assert_eq!(labels(&tree).first().map(String::as_str), Some("▾ a"));
        tree.toggle_folder();
        assert_eq!(labels(&tree).first().map(String::as_str), Some("▸ a"));
        tree.choose_next();
        tree.choose_next();
        let before = labels(&tree);
        tree.toggle_folder();
        assert_eq!(labels(&tree), before);
    }

    #[test]
    fn a_document_far_down_is_on_screen_when_the_tree_opens() {
        let files: Vec<String> = (10..20).map(|n| format!("{n}.md")).collect();
        let mut tree = FileTree::over(files, Some("18.md"));
        assert_eq!(tree.first_shown(4), 5);
    }

    #[test]
    fn a_shorter_tree_does_not_leave_empty_rows_below_it() {
        let mut tree = tree(&["a/1.md", "a/2.md", "a/3.md", "a/4.md"], Some("a/4.md"));
        assert_eq!(tree.first_shown(2), 3);
        for _ in 0..4 {
            tree.choose_previous();
        }
        tree.close_folder();
        assert_eq!(tree.first_shown(2), 0);
    }
}
