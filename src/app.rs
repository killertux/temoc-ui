//! The application itself, independent of any real terminal.

use crate::page::Page;
use crate::render::Document;
use crate::search::Search;
use crate::{NotImplemented, workspace};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect, Size};
use ratatui::style::{Modifier, Style};
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
    running: bool,
    workspace: PathBuf,
    page: Option<Page>,
    /// The pages behind the current one, the most recent last.
    back: Vec<Page>,
    /// The pages left by going back, the most recent last.
    forward: Vec<Page>,
    search: Option<Search>,
    /// Whether search shows only test files. Outlives the search box.
    only_tests: bool,
    status: Option<String>,
    /// The room for a page, as last drawn.
    page_size: Size,
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
            running: true,
            workspace: opening.workspace,
            page: None,
            back: Vec::new(),
            forward: Vec::new(),
            search: None,
            only_tests: false,
            status: None,
            page_size: Size::default(),
        };
        match opening.file {
            Some(file) => app.visit(&file, None).map_err(StartError::Refused)?,
            None => app.open_search(),
        }
        Ok(app)
    }

    /// Goes to a file of the workspace, leaving the current page behind.
    fn visit(&mut self, file: &str, anchor: Option<&str>) -> Result<(), String> {
        let mut page = self.load(file)?;
        if let Some(anchor) = anchor {
            page.go_to_anchor(anchor);
        }
        self.back.extend(self.page.replace(page));
        self.forward.clear();
        Ok(())
    }

    fn load(&self, file: &str) -> Result<Page, String> {
        let refuse = |error: &dyn Display| format!("Cannot read {file}: {error}");
        let text = fs::read_to_string(self.workspace.join(file)).map_err(|e| refuse(&e))?;
        let document = Document::parse(&text).map_err(|e| refuse(&e))?;
        Ok(Page::new(file.to_owned(), document, self.page_size))
    }

    fn go_back(&mut self) {
        if let Some(page) = self.back.pop() {
            self.forward.extend(self.page.replace(page));
        }
    }

    fn go_forward(&mut self) {
        if let Some(page) = self.forward.pop() {
            self.back.extend(self.page.replace(page));
        }
    }

    fn follow_focused_link(&mut self) -> Result<(), String> {
        let Some(page) = &self.page else {
            return Ok(());
        };
        let Some(link) = page.focused_link() else {
            return Ok(());
        };
        let place = workspace::follow_link(&self.workspace, page.file(), &link.target)?;
        self.visit(&place.file, place.anchor.as_deref())
    }

    fn open_search(&mut self) {
        self.search = Some(Search::open(&self.workspace, self.only_tests));
    }

    /// Opens the chosen result, if there is one.
    fn open_chosen_result(&mut self) -> Result<(), String> {
        let Some(hit) = self.search.as_ref().and_then(Search::chosen).cloned() else {
            return Ok(());
        };
        self.search = None;
        let anchor = hit.heading.map(|heading| heading.anchor);
        self.visit(&hit.file, anchor.as_deref())
    }

    /// Shows what went wrong, if anything did.
    fn report(&mut self, result: Result<(), String>) {
        self.status = result.err();
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
        self.status = None;
        if control && key.code == KeyCode::Char('c') {
            self.running = false;
            return Ok(());
        }
        if self.search.is_some() {
            self.handle_search_key(key.code, control);
            return Ok(());
        }
        if control {
            return Err(NotImplemented);
        }
        self.handle_page_key(key.code)
    }

    fn handle_search_key(&mut self, code: KeyCode, control: bool) {
        if code == KeyCode::Esc {
            self.search = None;
        } else if code == KeyCode::Enter {
            let result = self.open_chosen_result();
            self.report(result);
        } else if let Some(search) = &mut self.search {
            match (code, control) {
                (KeyCode::Down, _) => search.choose_next(),
                (KeyCode::Up, _) => search.choose_previous(),
                (KeyCode::Backspace, _) => search.delete_character(),
                (KeyCode::Char('t'), true) => {
                    search.toggle_only_tests();
                    self.only_tests = search.only_tests();
                }
                (KeyCode::Char(character), false) => search.type_character(character),
                _ => {}
            }
        }
    }

    fn handle_page_key(&mut self, code: KeyCode) -> Result<(), NotImplemented> {
        match code {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Char('/') => self.open_search(),
            KeyCode::Char('h') | KeyCode::Backspace => self.go_back(),
            KeyCode::Char('l') => self.go_forward(),
            KeyCode::Enter => {
                let result = self.follow_focused_link();
                self.report(result);
            }
            code => return self.move_on_page(code),
        }
        Ok(())
    }

    /// Reacts to the keys that move around the open page. With no page open,
    /// there is nothing for them to do.
    fn move_on_page(&mut self, code: KeyCode) -> Result<(), NotImplemented> {
        let act: fn(&mut Page) = match code {
            KeyCode::Char('j') | KeyCode::Down => |page| page.scroll_down(1),
            KeyCode::Char('k') | KeyCode::Up => |page| page.scroll_up(1),
            KeyCode::PageDown | KeyCode::Char(' ') => Page::page_down,
            KeyCode::PageUp => Page::page_up,
            KeyCode::Char('g') => Page::top,
            KeyCode::Char('G') => Page::bottom,
            KeyCode::Tab => Page::next_link,
            KeyCode::BackTab => Page::previous_link,
            KeyCode::Char(']') => Page::next_table,
            KeyCode::Char('[') => Page::previous_table,
            _ => return Err(NotImplemented),
        };
        if let Some(page) = &mut self.page {
            act(page);
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

    /// Draws the current screen, and adapts to its size.
    pub fn draw(&mut self, frame: &mut Frame) {
        let [body, status] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());
        self.page_size = body.as_size();
        if let Some(page) = &mut self.page {
            page.fit(self.page_size);
            frame.render_widget(Paragraph::new(page.lines()), body);
        }
        if let Some(message) = &self.status {
            frame.render_widget(Paragraph::new(message.as_str()), status);
        }
        if let Some(search) = &self.search {
            draw_search(search, frame);
        }
    }

    /// Takes the bytes waiting to be written to the terminal, such as a
    /// clipboard escape sequence.
    pub fn take_output(&mut self) -> Vec<u8> {
        Vec::new()
    }

    /// A snapshot of the application.
    #[must_use]
    pub fn state(&self) -> AppState {
        let page = self.page.as_ref();
        AppState {
            running: self.running,
            document: page.map(|page| page.file().to_owned()),
            status: self.status.clone(),
            focused_link: page
                .and_then(Page::focused_link)
                .map(|link| link.text.clone()),
            search_open: self.search.is_some(),
            search_results: self.search.as_ref().map_or_else(Vec::new, |search| {
                search.results().iter().map(ToString::to_string).collect()
            }),
            focused_table: page.and_then(Page::focused_table).map(String::from),
            ..AppState::default()
        }
    }
}

fn draw_search(search: &Search, frame: &mut Frame) {
    let area = frame.area().inner(Margin::new(4, 2));
    // The border takes two rows, and the search box another.
    let room = usize::from(area.height).saturating_sub(3);
    let first = search.selected().saturating_sub(room.saturating_sub(1));
    let results = search.results().iter().enumerate().skip(first).take(room);
    let lines: Vec<Line> = std::iter::once(Line::from(format!("/{}", search.query())))
        .chain(results.map(|(index, hit)| {
            let line = Line::from(hit.to_string());
            if index == search.selected() {
                line.style(Style::new().add_modifier(Modifier::REVERSED))
            } else {
                line
            }
        }))
        .collect();
    let title = if search.only_tests() {
        "Search test files"
    } else {
        "Search"
    };
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered().title(title)),
        area,
    );
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
        let key = Event::Key(KeyEvent::from(KeyCode::Char('r')));
        assert_eq!(app.handle_event(&key), Err(NotImplemented));
    }

    fn press(app: &mut App, code: KeyCode) {
        app.handle_event(&Event::Key(KeyEvent::from(code))).unwrap();
    }

    #[test]
    fn a_folder_without_a_page_can_close_search_and_still_take_keys() {
        let mut app = App::start(&launch("docs", &["guide"])).unwrap();
        for code in [
            KeyCode::Esc,
            KeyCode::Char('j'),
            KeyCode::Tab,
            KeyCode::Enter,
            KeyCode::Char('h'),
            KeyCode::Char(']'),
        ] {
            press(&mut app, code);
        }
        assert_eq!(app.state().document, None);
        assert!(app.state().running);
    }

    #[test]
    fn the_next_key_clears_the_status() {
        let mut app = App::start(&launch("docs", &[])).unwrap();
        press(&mut app, KeyCode::BackTab);
        press(&mut app, KeyCode::BackTab);
        press(&mut app, KeyCode::Enter);
        assert_eq!(
            app.state().status.as_deref(),
            Some("Link target not found: missing.md")
        );
        press(&mut app, KeyCode::Char('j'));
        assert_eq!(app.state().status, None);
    }

    #[test]
    fn a_search_result_that_disappeared_says_so() {
        let mut app = App::start(&launch("docs", &[])).unwrap();
        let result = app.visit("gone.md", None);
        assert!(result.is_err_and(|message| message.starts_with("Cannot read gone.md: ")));
        assert_eq!(app.state().document.as_deref(), Some("README.md"));
    }

    #[test]
    fn keys_with_no_meaning_in_search_do_nothing() {
        let mut app = App::start(&launch("docs", &[])).unwrap();
        press(&mut app, KeyCode::Char('/'));
        press(&mut app, KeyCode::Tab);
        assert!(app.state().search_open);
    }

    #[test]
    fn a_resized_terminal_is_not_a_missing_feature() {
        let mut app = App::start(&launch("docs", &[])).unwrap();
        assert_eq!(app.handle_event(&Event::Resize(100, 40)), Ok(()));
    }
}
