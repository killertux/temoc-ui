//! The application itself, independent of any real terminal.

use crossterm::event::Event;
use ratatui::Frame;
use ratatui::layout::Rect;
use std::path::PathBuf;

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
    pub fn start(_launch: &Launch) -> Result<Self, StartError> {
        Err(StartError::NotImplemented)
    }

    /// Reacts to a key press or mouse action.
    pub fn handle_event(&mut self, _event: &Event) {}

    /// Picks up the progress of background work, such as a test run.
    pub fn update(&mut self) {}

    /// Whether background work, such as a test run, is still going.
    #[must_use]
    pub fn is_busy(&self) -> bool {
        false
    }

    /// Draws the current screen.
    pub fn draw(&self, _frame: &mut Frame) {}

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
