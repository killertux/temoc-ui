//! The application itself, independent of any real terminal.

use crate::render::Document;
use crate::{NotImplemented, temoc, workspace};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::{Margin, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Paragraph};
use std::fmt::Display;
use std::fs;
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
    workspace: PathBuf,
    page: Option<Document>,
}

impl App {
    /// Starts the application.
    ///
    /// # Errors
    ///
    /// Returns [`StartError::Refused`] when the path given cannot be opened.
    /// The `--config` option is not built yet, and returns
    /// [`StartError::NotImplemented`].
    pub fn start(launch: &Launch) -> Result<Self, StartError> {
        let path = match launch.arguments.as_slice() {
            [] => None,
            [path] if !path.starts_with('-') => Some(path.as_str()),
            _ => return Err(StartError::NotImplemented),
        };
        let opening = workspace::locate(&launch.current_dir, path).map_err(StartError::Refused)?;
        let mut app = Self {
            state: AppState {
                running: true,
                ..AppState::default()
            },
            workspace: opening.workspace,
            page: None,
        };
        match opening.file {
            Some(file) => app.open(&file).map_err(StartError::Refused)?,
            None => app.open_search(),
        }
        Ok(app)
    }

    /// Opens a file of the workspace.
    fn open(&mut self, file: &str) -> Result<(), String> {
        let refuse = |error: &dyn Display| format!("Cannot read {file}: {error}");
        let text = fs::read_to_string(self.workspace.join(file)).map_err(|e| refuse(&e))?;
        let tests = temoc::find_tests(&text).map_err(|e| refuse(&e))?;
        self.page = Some(Document::parse(&text).map_err(|e| refuse(&e))?);
        self.state.document = Some(file.to_owned());
        self.state.focused_table = tests.tables.into_iter().next();
        Ok(())
    }

    fn open_search(&mut self) {
        self.state.search_open = true;
        self.state.search_results = workspace::markdown_files(&self.workspace);
    }

    /// Reacts to a key press or mouse action.
    ///
    /// # Errors
    ///
    /// Fails for input that is not built yet, so that a scenario using it
    /// cannot pass because nothing happened.
    pub fn handle_event(&mut self, event: &Event) -> Result<(), NotImplemented> {
        match event {
            Event::Key(key) if key.kind != KeyEventKind::Release => self.handle_key(key),
            Event::Mouse(_) => Err(NotImplemented),
            _ => Ok(()),
        }
    }

    fn handle_key(&mut self, key: &KeyEvent) -> Result<(), NotImplemented> {
        let control = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Char('c') if control => self.state.running = false,
            KeyCode::Char('q') if !control && !self.state.search_open => {
                self.state.running = false;
            }
            KeyCode::Char('/') if !control && !self.state.search_open => self.open_search(),
            _ => return Err(NotImplemented),
        }
        Ok(())
    }

    /// Picks up the progress of background work, such as a test run.
    pub fn update(&mut self) {}

    /// Whether background work, such as a test run, is still going.
    #[must_use]
    pub fn is_busy(&self) -> bool {
        false
    }

    /// Draws the current screen.
    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        if let Some(page) = &self.page {
            frame.render_widget(Paragraph::new(page.lines(area.width)), area);
        }
        if self.state.search_open {
            self.draw_search(frame);
        }
    }

    fn draw_search(&self, frame: &mut Frame) {
        let area = frame.area().inner(Margin::new(4, 2));
        let results = self.state.search_results.iter();
        let lines: Vec<Line> = std::iter::once(Line::from("/"))
            .chain(results.map(|result| Line::from(result.as_str())))
            .collect();
        frame.render_widget(Clear, area);
        frame.render_widget(
            Paragraph::new(lines).block(Block::bordered().title("Search")),
            area,
        );
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

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
        for arguments in [&["--config", "a.toml"][..], &["a.md", "--config", "b.toml"]] {
            let result = App::start(&launch("docs", arguments));
            assert_eq!(result.err(), Some(StartError::NotImplemented));
        }
    }

    #[test]
    fn a_key_that_is_not_built_says_so() {
        let mut app = App::start(&launch("docs", &[])).unwrap();
        let key = Event::Key(KeyEvent::from(KeyCode::Char('j')));
        assert_eq!(app.handle_event(&key), Err(NotImplemented));
    }

    #[test]
    fn a_resized_terminal_is_not_a_missing_feature() {
        let mut app = App::start(&launch("docs", &[])).unwrap();
        assert_eq!(app.handle_event(&Event::Resize(100, 40)), Ok(()));
    }
}
