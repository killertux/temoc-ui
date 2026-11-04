//! The application itself, independent of any real terminal.

use crate::{NotImplemented, temoc};
use crossterm::event::Event;
use ratatui::Frame;
use ratatui::layout::Rect;
use std::path::{Component, Path, PathBuf};

/// How temoc-ui was started.
#[derive(Debug, Clone, Default)]
pub struct Launch {
    /// The folder temoc-ui was started from.
    pub current_dir: PathBuf,
    /// The command line arguments, without the program name.
    pub arguments: Vec<String>,
    /// Folders to search ahead of `PATH` when starting temoc.
    pub extra_path: Vec<PathBuf>,
}

/// Why temoc-ui did not start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartError {
    /// The application has not been built yet.
    NotImplemented,
    /// temoc-ui refused to open, with a message for the user.
    Refused(String),
}

/// What the graph view currently shows.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GraphState {
    /// The files in each column, from left to right.
    pub columns: Vec<Vec<String>>,
    /// Every link in the graph, as `(from, to)`.
    pub links: Vec<(String, String)>,
    /// The selected file.
    pub selected: Option<String>,
    /// Links from the start page to the selected file, if it can be reached.
    pub distance: Option<usize>,
    /// The panel's list of links out of the selected file.
    pub links_out: Vec<String>,
    /// The panel's list of links into the selected file.
    pub links_in: Vec<String>,
    /// The part of the screen holding the graph, without the panel.
    pub area: Rect,
}

/// A snapshot of the application, for anything that needs to observe it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppState {
    /// `false` once the application has quit.
    pub running: bool,
    /// Path of the open document within the workspace.
    pub document: Option<String>,
    /// The status bar message.
    pub status: Option<String>,
    /// The text of the focused link.
    pub focused_link: Option<String>,
    /// Whether the search box is showing.
    pub search_open: bool,
    /// The search results, in order.
    pub search_results: Vec<String>,
    /// The name of the focused test table.
    pub focused_table: Option<String>,
    /// Each table's name and result, in document order.
    pub table_results: Vec<(String, String)>,
    /// The result for the whole document.
    pub file_result: Option<String>,
    /// The failures listed on screen.
    pub failures: Vec<String>,
    /// The arguments of the last temoc command.
    pub temoc_arguments: Option<Vec<String>>,
    /// The rows of the file tree, when it is showing.
    pub tree: Option<Vec<String>>,
    /// The selected row of the file tree.
    pub tree_selection: Option<String>,
    /// The graph view, when it is showing.
    pub graph: Option<GraphState>,
    /// The selected text.
    pub selection: Option<String>,
}

/// The temoc-ui application.
#[derive(Debug)]
pub struct App {
    state: AppState,
}

impl App {
    /// Starts the application.
    ///
    /// # Errors
    ///
    /// Returns [`StartError::Refused`] when the path given cannot be opened.
    /// Only opening a markdown file inside the current folder is built so far,
    /// anything else returns [`StartError::NotImplemented`].
    pub fn start(launch: &Launch) -> Result<Self, StartError> {
        let [path] = launch.arguments.as_slice() else {
            return Err(StartError::NotImplemented);
        };
        let file = launch.current_dir.join(path);
        if !is_inside(path) || !is_markdown(&file) || !file.is_file() {
            return Err(StartError::NotImplemented);
        }
        let tests = temoc::inspect_file(&file)
            .map_err(|error| StartError::Refused(format!("Cannot read {path}: {error}")))?;
        Ok(Self {
            state: AppState {
                running: true,
                document: Some(path.clone()),
                focused_table: tests.tables.into_iter().next(),
                ..AppState::default()
            },
        })
    }

    /// Reacts to a key press or mouse action.
    ///
    /// # Errors
    ///
    /// Fails until input is built, so that a scenario pressing keys cannot pass
    /// because nothing happened.
    pub fn handle_event(&mut self, _event: &Event) -> Result<(), NotImplemented> {
        Err(NotImplemented)
    }

    /// Picks up the progress of background work, such as a test run.
    pub fn update(&mut self) {}

    /// Whether background work, such as a test run, is still going.
    #[must_use]
    pub fn is_busy(&self) -> bool {
        false
    }

    /// Draws the current screen.
    ///
    /// # Errors
    ///
    /// Fails until the screen is built, so that text cannot pass as hidden
    /// because nothing was drawn.
    pub fn draw(&self, _frame: &mut Frame) -> Result<(), NotImplemented> {
        Err(NotImplemented)
    }

    /// Takes the bytes waiting to be written to the terminal, such as a
    /// clipboard escape sequence.
    pub fn take_output(&mut self) -> Vec<u8> {
        Vec::new()
    }

    /// A snapshot of the application.
    #[must_use]
    pub fn state(&self) -> AppState {
        self.state.clone()
    }
}

fn is_inside(path: &str) -> bool {
    Path::new(path)
        .components()
        .all(|part| matches!(part, Component::Normal(_)))
}

fn is_markdown(file: &Path) -> bool {
    file.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn launch(workspace: &str, arguments: &[&str]) -> Launch {
        Launch {
            current_dir: Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("samples")
                .join(workspace),
            arguments: arguments.iter().map(ToString::to_string).collect(),
            extra_path: Vec::new(),
        }
    }

    #[test]
    fn opening_a_test_file_focuses_its_first_table() {
        let state = App::start(&launch("calculator", &["passing.md"]))
            .unwrap()
            .state();
        assert!(state.running);
        assert_eq!(state.document.as_deref(), Some("passing.md"));
        assert_eq!(state.focused_table.as_deref(), Some("Addition"));
    }

    #[test]
    fn what_is_not_built_says_so() {
        for arguments in [&[][..], &["nope.md"], &["../docs/README.md"], &["data.txt"]] {
            let result = App::start(&launch("docs", arguments));
            assert_eq!(result.err(), Some(StartError::NotImplemented));
        }
    }
}
