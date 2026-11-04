//! The application itself, independent of any real terminal.

use crate::page::Page;
use crate::render::Document;
use crate::run::{FileResults, Request, Run};
use crate::search::Search;
use crate::temoc::{RunReport, RunResult, TestTable};
use crate::tree::{FileTree, Row};
use crate::{NotImplemented, workspace};
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Position, Rect, Size};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Paragraph};
use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::path::PathBuf;

/// The configuration file temoc-ui looks for in the workspace.
const CONFIGURATION: &str = "temoc.toml";
/// The result of a file that has no results yet.
const NOT_RUN: &str = "not run";
/// How many lines one step of the mouse wheel scrolls.
const WHEEL_STEP: usize = 3;

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

/// The arguments of the command line, once they are understood.
#[derive(Debug, Default, PartialEq, Eq)]
struct Arguments<'launch> {
    path: Option<&'launch str>,
    config: Option<&'launch str>,
}

impl<'launch> Arguments<'launch> {
    fn read(arguments: &'launch [String]) -> Result<Self, String> {
        let mut read = Self::default();
        let mut arguments = arguments.iter().map(String::as_str);
        while let Some(argument) = arguments.next() {
            if argument == "--config" {
                read.config = Some(arguments.next().ok_or("--config needs a file")?);
            } else if argument.starts_with('-') {
                return Err(format!("Unknown option: {argument}"));
            } else if read.path.replace(argument).is_some() {
                return Err("Only one path can be opened".into());
            }
        }
        Ok(read)
    }
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
    tree: Option<FileTree>,
    /// Whether search shows only test files. Outlives the search box.
    only_tests: bool,
    status: Option<String>,
    /// The room for a page, as last drawn.
    page_size: Size,
    /// The whole screen, as last drawn.
    screen: Rect,
    /// Where the left mouse button went down, if this can still be a click.
    pressed: Option<Position>,
    /// The folder temoc-ui was started from.
    start_folder: PathBuf,
    /// The configuration file given with `--config`, as written.
    config: Option<String>,
    /// Folders to search ahead of `PATH` when starting temoc.
    extra_path: Vec<PathBuf>,
    /// The temoc process at work.
    run: Option<Run>,
    /// What each file's runs have reported. Outlives the pages.
    results: HashMap<String, FileResults>,
    /// The arguments of the last temoc command.
    temoc_arguments: Option<Vec<String>>,
}

impl App {
    /// Starts the application.
    ///
    /// # Errors
    ///
    /// Fails with a message for the user when the arguments make no sense or
    /// the path given cannot be opened.
    pub fn start(launch: &Launch) -> Result<Self, String> {
        let arguments = Arguments::read(&launch.arguments)?;
        let opening = workspace::locate(&launch.current_dir, arguments.path)?;
        let mut app = Self {
            running: true,
            workspace: opening.workspace,
            page: None,
            back: Vec::new(),
            forward: Vec::new(),
            search: None,
            tree: None,
            only_tests: false,
            status: None,
            page_size: Size::default(),
            screen: Rect::default(),
            pressed: None,
            start_folder: launch.current_dir.clone(),
            config: arguments.config.map(String::from),
            extra_path: launch.extra_path.clone(),
            run: None,
            results: HashMap::new(),
            temoc_arguments: None,
        };
        match opening.file {
            Some(file) => app.visit(&file, None)?,
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
        let mut page = Page::new(file.to_owned(), document, self.page_size);
        page.show_results(self.results.get(file));
        Ok(page)
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

    fn open_tree(&mut self) {
        let document = self.page.as_ref().map(Page::file);
        self.tree = Some(FileTree::open(&self.workspace, document));
    }

    /// Opens the selected file, or else the selected folder.
    fn open_tree_selection(&mut self) -> Result<(), String> {
        let Some(tree) = &mut self.tree else {
            return Ok(());
        };
        let Some(file) = tree.chosen_file().map(String::from) else {
            tree.open_folder();
            return Ok(());
        };
        self.tree = None;
        self.visit(&file, None)
    }

    /// Runs every table of the open file.
    fn run_file(&mut self) -> Result<(), String> {
        let page = self.page.as_ref().filter(|page| page.is_test_file());
        let page = page.ok_or("This file has no temoc tests")?;
        let (file, tables) = (page.file().to_owned(), page.tables().to_vec());
        self.start_run(&file, file.clone(), &tables)
    }

    /// Runs the focused table.
    fn run_table(&mut self) -> Result<(), String> {
        let page = self.page.as_ref();
        let found = page.and_then(|page| Some((page.file(), page.focused_table()?)));
        let (file, table) = found.ok_or("No test table to run")?;
        let target = format!("{file}:{}", table.directive_line);
        let (file, table) = (file.to_owned(), table.clone());
        self.start_run(&file, target, &[table])
    }

    fn start_run(
        &mut self,
        file: &str,
        target: String,
        tables: &[TestTable],
    ) -> Result<(), String> {
        if self.run.is_some() {
            return Err("temoc is already running".into());
        }
        let arguments = vec!["-c".into(), self.configuration()?, "-s".into(), target];
        self.temoc_arguments = Some(arguments.clone());
        let request = Request {
            folder: self.workspace.clone(),
            extra_path: self.extra_path.clone(),
            arguments,
            file: file.to_owned(),
            tables: tables.to_vec(),
        };
        match Run::start(request) {
            Ok(run) => self.run = Some(run),
            Err(message) => self.keep_report(file, tables, RunReport::stopped(message)),
        }
        Ok(())
    }

    /// The configuration file to give temoc, which runs in the workspace.
    fn configuration(&self) -> Result<String, String> {
        let Some(written) = &self.config else {
            return if self.workspace.join(CONFIGURATION).is_file() {
                Ok(CONFIGURATION.into())
            } else {
                Err(format!(
                    "No temoc configuration found. Add {CONFIGURATION} or use --config."
                ))
            };
        };
        let missing = || format!("Configuration file not found: {written}");
        let path = self.start_folder.join(written);
        let path = path.canonicalize().map_err(|_| missing())?;
        if !path.is_file() {
            return Err(missing());
        }
        let workspace = self.workspace.canonicalize();
        let inside = workspace.map(|workspace| path.strip_prefix(workspace).map(PathBuf::from));
        let path = match inside {
            Ok(Ok(inside)) => inside,
            _ => path,
        };
        Ok(path.to_string_lossy().into_owned())
    }

    /// Keeps what a run reported, and says so if it needs saying.
    fn keep_report(&mut self, file: &str, tables: &[TestTable], report: RunReport) {
        match (report.result, &report.error) {
            (RunResult::Error, Some(message)) => {
                self.status = Some(format!("temoc failed: {message}"));
            }
            (RunResult::Error, None) => self.status = Some("temoc failed".into()),
            (RunResult::NoTests, _) => self.status = Some("temoc found nothing to run".into()),
            _ => {}
        }
        let results = self.results.entry(file.to_owned()).or_default();
        results.record(tables, report);
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
            Event::Mouse(mouse) => {
                self.handle_mouse(*mouse);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        let spot = Position::new(mouse.column, mouse.row);
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => self.pressed = Some(spot),
            MouseEventKind::Drag(MouseButton::Left) if self.pressed != Some(spot) => {
                self.pressed = None;
            }
            MouseEventKind::Up(MouseButton::Left) if self.pressed.take() == Some(spot) => {
                self.status = None;
                self.click(spot);
            }
            kind @ (MouseEventKind::ScrollDown
            | MouseEventKind::ScrollUp
            | MouseEventKind::ScrollLeft
            | MouseEventKind::ScrollRight) => {
                self.status = None;
                self.turn_wheel(kind, mouse.modifiers.contains(KeyModifiers::SHIFT));
            }
            _ => {}
        }
    }

    fn click(&mut self, spot: Position) {
        if self.search.is_some() {
            return;
        }
        if self.tree.is_some() {
            self.click_tree(spot);
        } else if self
            .page
            .as_mut()
            .is_some_and(|page| page.click(spot.x, spot.y))
        {
            let result = self.follow_focused_link();
            self.report(result);
        }
    }

    fn click_tree(&mut self, spot: Position) {
        let area = pop_up_area(self.screen);
        if !area.contains(spot) {
            self.tree = None;
            return;
        }
        let rows = area.inner(Margin::new(1, 1));
        let Some(tree) = self.tree.as_mut().filter(|_| rows.contains(spot)) else {
            return;
        };
        if !tree.select_shown(usize::from(spot.y.saturating_sub(rows.y))) {
            return;
        }
        if tree.chosen_file().is_some() {
            let result = self.open_tree_selection();
            self.report(result);
        } else {
            tree.toggle_folder();
        }
    }

    /// Holding Shift turns an upright wheel into a sideways one.
    fn turn_wheel(&mut self, kind: MouseEventKind, shift: bool) {
        if self.search.is_some() {
            return;
        }
        if let Some(tree) = &mut self.tree {
            match (kind, shift) {
                (MouseEventKind::ScrollDown, false) => tree.scroll_down(WHEEL_STEP),
                (MouseEventKind::ScrollUp, false) => tree.scroll_up(WHEEL_STEP),
                _ => {}
            }
        } else if let Some(page) = &mut self.page {
            match (kind, shift) {
                (MouseEventKind::ScrollDown, false) => page.scroll_down(WHEEL_STEP),
                (MouseEventKind::ScrollUp, false) => page.scroll_up(WHEEL_STEP),
                (MouseEventKind::ScrollRight, _) | (MouseEventKind::ScrollDown, true) => {
                    page.scroll_right();
                }
                (MouseEventKind::ScrollLeft, _) | (MouseEventKind::ScrollUp, true) => {
                    page.scroll_left();
                }
                _ => {}
            }
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
        if self.tree.is_some() {
            if !control {
                self.handle_tree_key(key.code);
            }
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

    fn handle_tree_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc | KeyCode::Char('t') => self.tree = None,
            KeyCode::Char('q') => self.running = false,
            KeyCode::Enter => {
                let result = self.open_tree_selection();
                self.report(result);
            }
            code => {
                if let Some(tree) = &mut self.tree {
                    match code {
                        KeyCode::Char('j') | KeyCode::Down => tree.choose_next(),
                        KeyCode::Char('k') | KeyCode::Up => tree.choose_previous(),
                        KeyCode::Char('l') | KeyCode::Right => tree.open_folder(),
                        KeyCode::Char('h') | KeyCode::Left => tree.close_folder(),
                        _ => {}
                    }
                }
            }
        }
    }

    fn handle_page_key(&mut self, code: KeyCode) -> Result<(), NotImplemented> {
        match code {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Char('/') => self.open_search(),
            KeyCode::Char('t') => self.open_tree(),
            KeyCode::Char('h') | KeyCode::Backspace => self.go_back(),
            KeyCode::Char('l') => self.go_forward(),
            KeyCode::Enter => {
                let result = self.follow_focused_link();
                self.report(result);
            }
            KeyCode::Char('R') => {
                let result = self.run_file();
                self.report(result);
            }
            KeyCode::Char('r') => {
                let result = self.run_table();
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
            KeyCode::Right => Page::scroll_right,
            KeyCode::Left => Page::scroll_left,
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
    pub fn update(&mut self) {
        let Some(report) = self.run.as_mut().and_then(Run::finished) else {
            return;
        };
        if let Some(run) = self.run.take() {
            self.keep_report(run.file(), run.tables(), report);
        }
    }

    /// Whether the application is still open.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Whether background work, such as a test run, is still going.
    #[must_use]
    pub fn is_busy(&self) -> bool {
        self.run.is_some()
    }

    /// Draws the current screen, and adapts to its size.
    pub fn draw(&mut self, frame: &mut Frame) {
        self.screen = frame.area();
        let [body, status] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());
        self.page_size = body.as_size();
        if let Some(page) = &mut self.page {
            page.show_results(self.results.get(page.file()));
            page.fit(self.page_size);
            frame.render_widget(Paragraph::new(page.lines()), body);
        }
        self.draw_status(frame, status);
        if let Some(search) = &self.search {
            draw_search(search, frame);
        }
        if let Some(tree) = &mut self.tree {
            draw_tree(tree, frame);
        }
    }

    /// Draws the message on the left of the status bar, and the result of the
    /// open file on the right.
    fn draw_status(&self, frame: &mut Frame, area: Rect) {
        let running = self
            .run
            .as_ref()
            .map(|run| format!("Running {}...", run.file()));
        let result = self.file_result().filter(|result| *result != NOT_RUN);
        let result = result.map_or_else(String::new, |result| format!("file: {result}"));
        let width = Line::from(result.as_str()).width();
        let width = u16::try_from(width).unwrap_or(area.width);
        let [message, file] =
            Layout::horizontal([Constraint::Min(0), Constraint::Length(width)]).areas(area);
        if let Some(text) = self.status.as_ref().or(running.as_ref()) {
            frame.render_widget(Paragraph::new(text.as_str()), message);
        }
        frame.render_widget(Paragraph::new(result), file);
    }

    /// The result of the open file, if it has tests.
    fn file_result(&self) -> Option<&'static str> {
        let page = self.page.as_ref().filter(|page| page.is_test_file())?;
        let results = self.results.get(page.file());
        Some(results.map_or(NOT_RUN, |results| results.summary(page.tables())))
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
        let results = page
            .and_then(|page| self.results.get(page.file()))
            .cloned()
            .unwrap_or_default();
        let tables = page.map_or(&[][..], Page::tables);
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
            focused_table: page
                .and_then(Page::focused_table)
                .map(|table| table.name.clone()),
            table_results: results.by_table(tables),
            file_result: self.file_result().map(String::from),
            failures: results.failures(tables),
            temoc_arguments: self.temoc_arguments.clone(),
            tree: self
                .tree
                .as_ref()
                .map(|tree| tree.rows().iter().map(Row::label).collect()),
            tree_selection: self
                .tree
                .as_ref()
                .and_then(FileTree::selection)
                .map(|row| row.name.clone()),
            ..AppState::default()
        }
    }
}

/// Where a pop-up goes.
fn pop_up_area(screen: Rect) -> Rect {
    screen.inner(Margin::new(4, 2))
}

fn highlighted(line: Line<'_>) -> Line<'_> {
    line.style(Style::new().add_modifier(Modifier::REVERSED))
}

fn draw_tree(tree: &mut FileTree, frame: &mut Frame) {
    let area = pop_up_area(frame.area());
    // The border takes two rows.
    let room = usize::from(area.height).saturating_sub(2);
    let first = tree.first_shown(room);
    let selected = tree.selected();
    let rows = tree.rows().iter().enumerate().skip(first).take(room);
    let lines: Vec<Line> = rows
        .map(|(index, row)| {
            let line = Line::from(format!("{}{}", "  ".repeat(row.depth), row.label()));
            if index == selected {
                highlighted(line)
            } else {
                line
            }
        })
        .collect();
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered().title("Files")),
        area,
    );
}

fn draw_search(search: &Search, frame: &mut Frame) {
    let area = pop_up_area(frame.area());
    // The border takes two rows, and the search box another.
    let room = usize::from(area.height).saturating_sub(3);
    let first = search.selected().saturating_sub(room.saturating_sub(1));
    let results = search.results().iter().enumerate().skip(first).take(room);
    let lines: Vec<Line> = std::iter::once(Line::from(format!("/{}", search.query())))
        .chain(results.map(|(index, hit)| {
            let line = Line::from(hit.to_string());
            if index == search.selected() {
                highlighted(line)
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
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Cell;
    use std::fs::Permissions;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use std::thread::sleep;
    use std::time::Duration;
    use std::{env, process};

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
    fn the_config_option_can_come_before_or_after_the_path() {
        let path = Some("a.md");
        let config = Some("b.toml");
        for arguments in [
            ["a.md", "--config", "b.toml"],
            ["--config", "b.toml", "a.md"],
        ] {
            let arguments = arguments.map(String::from);
            assert_eq!(Arguments::read(&arguments), Ok(Arguments { path, config }));
        }
    }

    #[test]
    fn a_key_that_is_not_built_says_so() {
        let mut app = App::start(&launch("docs", &[])).unwrap();
        let key = Event::Key(KeyEvent::from(KeyCode::Char('v')));
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
    fn the_tree_opens_without_a_document() {
        let mut app = App::start(&launch("docs", &["guide"])).unwrap();
        press(&mut app, KeyCode::Esc);
        press(&mut app, KeyCode::Char('t'));
        let state = app.state();
        assert_eq!(state.tree_selection.as_deref(), Some("getting-started.md"));
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.state().document.as_deref(), Some("getting-started.md"));
    }

    #[test]
    fn t_is_typed_in_search() {
        let mut app = App::start(&launch("docs", &[])).unwrap();
        press(&mut app, KeyCode::Char('/'));
        press(&mut app, KeyCode::Char('t'));
        assert_eq!(app.state().tree, None);
    }

    #[test]
    fn q_quits_from_the_tree() {
        let mut app = App::start(&launch("docs", &[])).unwrap();
        press(&mut app, KeyCode::Char('t'));
        press(&mut app, KeyCode::Char('q'));
        assert!(!app.state().running);
    }

    #[test]
    fn the_page_behind_the_tree_takes_no_keys() {
        let mut app = App::start(&launch("calculator", &["passing.md"])).unwrap();
        press(&mut app, KeyCode::Char('t'));
        for key in ['/', 'R', 'r', ']', 'v', '?'] {
            press(&mut app, KeyCode::Char(key));
        }
        let state = app.state();
        assert!(!state.search_open);
        assert_eq!(state.temoc_arguments, None);
        assert_eq!(state.focused_table.as_deref(), Some("Addition"));
        assert!(state.tree.is_some());
    }

    #[test]
    fn control_keys_do_not_move_the_tree() {
        let mut app = App::start(&launch("docs", &[])).unwrap();
        press(&mut app, KeyCode::Char('t'));
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL);
        app.handle_event(&Event::Key(key)).unwrap();
        assert_eq!(app.state().tree_selection.as_deref(), Some("README.md"));
    }

    #[test]
    fn a_file_that_disappeared_says_so_and_closes_the_tree() {
        let workspace = Workspace::new("vanish", "true");
        let mut app = workspace.app();
        press(&mut app, KeyCode::Char('t'));
        press(&mut app, KeyCode::Char('k'));
        assert_eq!(app.state().tree_selection.as_deref(), Some("other.md"));
        fs::remove_file(workspace.0.join("other.md")).unwrap();
        press(&mut app, KeyCode::Enter);
        let state = app.state();
        assert_eq!(state.tree, None);
        assert_eq!(state.document.as_deref(), Some("README.md"));
        assert!(
            state
                .status
                .is_some_and(|s| s.starts_with("Cannot read other.md: "))
        );
    }

    fn mouse(app: &mut App, kind: MouseEventKind, column: u16, row: u16) {
        let event = Event::Mouse(MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        });
        app.handle_event(&event).unwrap();
    }

    fn click(app: &mut App, column: u16, row: u16) {
        let button = MouseButton::Left;
        mouse(app, MouseEventKind::Down(button), column, row);
        mouse(app, MouseEventKind::Up(button), column, row);
    }

    fn app_on_screen(arguments: &[&str]) -> App {
        let mut app = App::start(&launch("docs", arguments)).unwrap();
        status_bar(&mut app);
        app
    }

    /// Where a link of the docs README is on a 40 column screen.
    fn link_spot(app: &App, text: &str) -> (u16, u16) {
        let lines = app.page.as_ref().unwrap().lines();
        let (row, column) = lines
            .iter()
            .enumerate()
            .find_map(|(row, line)| Some((row, line.to_string().find(text)?)))
            .unwrap();
        (u16::try_from(column).unwrap(), u16::try_from(row).unwrap())
    }

    #[test]
    fn pressing_and_releasing_in_different_spots_is_not_a_click() {
        let mut app = app_on_screen(&[]);
        let (column, row) = link_spot(&app, "Reference");
        let button = MouseButton::Left;
        mouse(&mut app, MouseEventKind::Down(button), column, row);
        mouse(&mut app, MouseEventKind::Up(button), column + 1, row);
        assert_eq!(app.state().document.as_deref(), Some("README.md"));
    }

    #[test]
    fn dragging_away_and_back_is_not_a_click() {
        let mut app = app_on_screen(&[]);
        let (column, row) = link_spot(&app, "Reference");
        let button = MouseButton::Left;
        mouse(&mut app, MouseEventKind::Down(button), column, row);
        mouse(&mut app, MouseEventKind::Drag(button), column + 1, row);
        mouse(&mut app, MouseEventKind::Drag(button), column, row);
        mouse(&mut app, MouseEventKind::Up(button), column, row);
        assert_eq!(app.state().document.as_deref(), Some("README.md"));
    }

    #[test]
    fn other_buttons_do_not_click() {
        let mut app = app_on_screen(&[]);
        let (column, row) = link_spot(&app, "Reference");
        let button = MouseButton::Right;
        mouse(&mut app, MouseEventKind::Down(button), column, row);
        mouse(&mut app, MouseEventKind::Up(button), column, row);
        assert_eq!(app.state().document.as_deref(), Some("README.md"));
    }

    #[test]
    fn moving_the_mouse_leaves_the_status_alone() {
        let mut app = app_on_screen(&[]);
        press(&mut app, KeyCode::BackTab);
        press(&mut app, KeyCode::BackTab);
        press(&mut app, KeyCode::Enter);
        mouse(&mut app, MouseEventKind::Moved, 5, 5);
        assert!(app.state().status.is_some());
        mouse(&mut app, MouseEventKind::ScrollDown, 5, 5);
        assert_eq!(app.state().status, None);
    }

    #[test]
    fn the_mouse_does_nothing_while_search_is_open() {
        let mut app = app_on_screen(&["guide/long-page.md"]);
        press(&mut app, KeyCode::Char('/'));
        mouse(&mut app, MouseEventKind::ScrollDown, 0, 0);
        click(&mut app, 0, 0);
        assert!(app.state().search_open);
        press(&mut app, KeyCode::Esc);
        status_bar(&mut app);
        assert_eq!(
            app.page
                .as_ref()
                .unwrap()
                .lines()
                .first()
                .map(ToString::to_string),
            Some("Long Page".to_owned())
        );
    }

    #[test]
    fn clicking_the_border_of_the_tree_keeps_it_open() {
        let mut app = app_on_screen(&[]);
        press(&mut app, KeyCode::Char('t'));
        status_bar(&mut app);
        let area = pop_up_area(app.screen);
        click(&mut app, area.x, area.y);
        click(&mut app, area.x + 5, area.bottom() - 1);
        let state = app.state();
        assert!(state.tree.is_some());
        assert_eq!(state.tree_selection.as_deref(), Some("README.md"));
    }

    #[test]
    fn clicking_below_the_rows_of_the_tree_keeps_it_open() {
        let mut app = app_on_screen(&[]);
        press(&mut app, KeyCode::Char('t'));
        status_bar(&mut app);
        let area = pop_up_area(app.screen);
        click(&mut app, area.x + 5, area.bottom() - 2);
        let state = app.state();
        assert!(state.tree.is_some());
        assert_eq!(state.tree_selection.as_deref(), Some("README.md"));
    }

    #[test]
    fn a_resized_terminal_is_not_a_missing_feature() {
        let mut app = App::start(&launch("docs", &[])).unwrap();
        assert_eq!(app.handle_event(&Event::Resize(100, 40)), Ok(()));
    }

    /// A workspace of its own, with a `temoc` that does what `script` says.
    struct Workspace(PathBuf);

    impl Workspace {
        fn new(name: &str, script: &str) -> Self {
            let folder = env::temp_dir().join(format!("temoc-ui-{name}-{}", process::id()));
            fs::create_dir_all(&folder).unwrap();
            let table = "[//]: # \"decisionTable A\"\n\n| a |\n| - |\n| 1 |\n";
            fs::write(folder.join("README.md"), format!("# Name\n\n{table}")).unwrap();
            fs::write(folder.join("other.md"), table).unwrap();
            fs::write(folder.join(CONFIGURATION), "").unwrap();
            let temoc = folder.join("temoc");
            fs::write(&temoc, format!("#!/bin/sh\n{script}\n")).unwrap();
            fs::set_permissions(&temoc, Permissions::from_mode(0o755)).unwrap();
            Self(folder)
        }

        fn app(&self) -> App {
            App::start(&Launch {
                current_dir: self.0.clone(),
                arguments: Vec::new(),
                extra_path: vec![self.0.clone()],
            })
            .unwrap()
        }
    }

    impl Drop for Workspace {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    fn settle(app: &mut App) {
        while app.is_busy() {
            sleep(Duration::from_millis(5));
            app.update();
        }
    }

    fn status_bar(app: &mut App) -> String {
        let mut terminal = Terminal::new(TestBackend::new(40, 12)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let buffer = terminal.backend().buffer();
        let cells = (0..40).filter_map(|x| buffer.cell((x, 11)));
        let row: String = cells.map(Cell::symbol).collect();
        row.trim_end().to_owned()
    }

    #[test]
    fn a_run_leaves_the_viewer_free_and_refuses_another() {
        let workspace = Workspace::new("busy", "sleep 30");
        let mut app = workspace.app();
        press(&mut app, KeyCode::Char('R'));
        assert!(app.is_busy());
        assert_eq!(status_bar(&mut app), "Running README.md...");
        press(&mut app, KeyCode::Char('j'));
        press(&mut app, KeyCode::Char('r'));
        assert_eq!(
            app.state().status.as_deref(),
            Some("temoc is already running")
        );
        assert_eq!(status_bar(&mut app), "temoc is already running");
        press(&mut app, KeyCode::Char('j'));
        assert_eq!(status_bar(&mut app), "Running README.md...");
    }

    #[test]
    fn results_go_to_the_file_that_ran_even_after_leaving_it() {
        let workspace = Workspace::new("left", "sleep 0.3; echo 'Testing file a.md...OK'");
        let mut app = workspace.app();
        press(&mut app, KeyCode::Char('R'));
        app.visit("other.md", None).unwrap();
        assert!(app.is_busy());
        settle(&mut app);
        assert_eq!(app.state().file_result.as_deref(), Some("not run"));
        press(&mut app, KeyCode::Char('h'));
        assert_eq!(app.state().file_result.as_deref(), Some("passed"));
    }

    #[test]
    fn a_page_opened_after_a_run_shows_its_results() {
        let workspace = Workspace::new("reopen", "echo 'Testing file a.md...OK'");
        let mut app = workspace.app();
        press(&mut app, KeyCode::Char('R'));
        settle(&mut app);
        app.visit("other.md", None).unwrap();
        status_bar(&mut app);
        app.visit("README.md", None).unwrap();
        assert_eq!(app.state().file_result.as_deref(), Some("passed"));
        assert!(
            app.page
                .as_ref()
                .is_some_and(|page| page.lines().iter().any(|l| l.to_string() == "✓ passed"))
        );
    }

    #[test]
    fn a_run_that_found_nothing_says_so() {
        let workspace = Workspace::new("none", "echo 'Testing file a.md...NONE'");
        let mut app = workspace.app();
        press(&mut app, KeyCode::Char('r'));
        settle(&mut app);
        assert_eq!(
            app.state().status.as_deref(),
            Some("temoc found nothing to run")
        );
        assert_eq!(app.state().file_result.as_deref(), Some("not run"));
    }

    #[test]
    fn a_run_with_no_message_still_says_it_failed() {
        let workspace = Workspace::new("silent", "exit 3");
        let mut app = workspace.app();
        press(&mut app, KeyCode::Char('R'));
        settle(&mut app);
        assert_eq!(app.state().status.as_deref(), Some("temoc failed"));
    }

    #[test]
    fn a_config_outside_the_workspace_is_passed_as_a_full_path() {
        let workspace = Workspace::new("outside", "true");
        let root = workspace.0.canonicalize().unwrap();
        fs::create_dir(root.join("sub")).unwrap();
        fs::write(root.join("sub/README.md"), "# Sub").unwrap();
        let app = App::start(&Launch {
            current_dir: root.clone(),
            arguments: vec!["sub".into(), "--config".into(), CONFIGURATION.into()],
            extra_path: Vec::new(),
        })
        .unwrap();
        assert_eq!(
            app.configuration(),
            Ok(root.join(CONFIGURATION).to_string_lossy().into_owned())
        );
    }

    #[test]
    fn a_folder_is_not_a_configuration_file() {
        let mut app = App::start(&launch("calculator", &["--config", "."])).unwrap();
        app.visit("passing.md", None).unwrap();
        press(&mut app, KeyCode::Char('R'));
        assert_eq!(
            app.state().status.as_deref(),
            Some("Configuration file not found: .")
        );
    }
}
