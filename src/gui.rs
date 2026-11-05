//! The graphical interface, built with egui. See `spec/gui/`.

mod icons;
mod look;
mod markdown;
mod pictures;
mod selection;
mod view;

use crate::app::{Arguments, Launch};
use crate::clipboard;
use crate::focus::Focus;
use crate::history::History;
use crate::search::Search;
use crate::session::{Opened, Session, Start};
use crate::temoc::{Failure, RunResult, TestTable};
use crate::tree::FileTree;
use crate::workspace;
pub use eframe::egui;
use egui::accesskit::Role;
use egui::{
    Align, Area, Button, CentralPanel, Color32, Context, Event, Frame, Id, Key, Label, Layout,
    Modifiers, OpenUrl, Order, Panel, PointerButton, Response, RichText, ScrollArea, Sense,
    TextEdit, Theme, Ui, UiBuilder, Vec2, ViewportCommand, WidgetInfo, vec2,
};
use icons::Icon;
use look::Palette;
use pictures::Pictures;
use std::collections::BTreeMap;
use std::time::Duration;
pub use view::Position;
use view::{Reach, Scroll, Sideways, View};

/// How big the window opens, as in the design.
const WINDOW: [f32; 2] = [1440.0, 900.0];
/// How often a run is looked at while nothing else happens.
const TICK: Duration = Duration::from_millis(100);
/// How far the page sits from its edges.
const PAGE_MARGIN: i8 = 40;
/// The widest the search box gets.
const SEARCH_WIDTH: f32 = 640.0;
/// The keys listed by help, as in `spec/gui/keys.md`.
const KEYS: [(&str, &str); 19] = [
    ("Up, Down", "Scroll up or down one line"),
    ("PageDown, Space", "Scroll down one page"),
    ("PageUp", "Scroll up one page"),
    ("Home, End", "Go to the top or bottom"),
    ("Left, Right", "Scroll a wide table or code sideways"),
    ("Tab, Shift-Tab", "Focus the next or previous link"),
    ("Enter", "Follow the focused link"),
    ("Alt-Left, Ctrl-[", "Go back"),
    ("Alt-Right, Ctrl-]", "Go forward"),
    ("Ctrl-k, /", "Open search"),
    ("Alt-Down, Alt-Up", "Focus the next or previous test table"),
    (
        "Alt-Shift-Down, Alt-Shift-Up",
        "Focus the next or previous test row",
    ),
    (
        "Ctrl-Enter",
        "Run the focused row, or else the focused table",
    ),
    ("Ctrl-r", "Run the whole file"),
    ("Ctrl-b", "Show or hide the file tree"),
    ("Ctrl-g", "Show or hide the graph"),
    ("Ctrl-c", "Copy the selected text"),
    ("F1, ?", "Show or hide help"),
    ("Ctrl-q", "Quit"),
];

/// What the fixture finds things by, in the description of a widget. Each one
/// says what the widget is to anyone who cannot see where it sits.
pub mod tags {
    /// The path of the open document.
    pub const DOCUMENT: &str = "document";
    /// The status bar message.
    pub const STATUS: &str = "status";
    /// The result for the whole document.
    pub const FILE_RESULT: &str = "file result";
    /// The banner above the page.
    pub const BANNER: &str = "banner";
    /// The heading of the file tree.
    pub const TREE: &str = "file tree";
    /// A file of the tree that holds tests.
    pub const TEST_FILE: &str = "test file";
    /// The heading of the tests panel.
    pub const TESTS_PANEL: &str = "tests panel";
    /// A table in the tests panel.
    pub const PANEL_TABLE: &str = "panel table";
    /// Something the tests panel lists as not passed.
    pub const NOT_PASSED: &str = "not passed";
    /// The last temoc command.
    pub const LAST_COMMAND: &str = "last command";
    /// The search box.
    pub const SEARCH: &str = "search";
    /// A result of the search.
    pub const SEARCH_RESULT: &str = "search result";
    /// The heading of the help.
    pub const HELP: &str = "help";
    /// The header of a test table, named after the table.
    pub const TEST_TABLE: &str = "test table";
    /// A test row, named after its line.
    pub const TEST_ROW: &str = "test row";
    /// The mark of a test row, written `line: mark`.
    pub const MARK: &str = "mark";
    /// A note under a test row, written `line: note`.
    pub const NOTE: &str = "note";
    /// What a test table says below itself.
    pub const VERDICT: &str = "verdict";
    /// The focused link.
    pub const FOCUSED_LINK: &str = "focused link";
}

/// Something asked for during a frame, carried out at the end of it.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Command {
    Open(String),
    Follow(String),
    FollowFocused,
    /// Scrolls to a heading or footnote on this line.
    GoToLine(usize),
    /// Puts text on the clipboard.
    Copy(String),
    NextLink,
    PreviousLink,
    Scroll(Scroll),
    Sideways(Sideways),
    ToggleFolder(usize),
    Back,
    Forward,
    OpenSearch,
    CloseSearch,
    OpenResult(usize),
    OnlyTestFiles,
    NextTable,
    PreviousTable,
    NextRow,
    PreviousRow,
    /// Focuses a table, and the row on a line of it if there is one.
    Focus {
        table: usize,
        line: Option<usize>,
    },
    RunFile,
    /// Runs the focused row, or else the focused table.
    RunFocused,
    /// Runs the row on a line.
    RunRow(usize),
    ToggleTree,
    ToggleTests,
    ToggleHelp,
    CloseHelp,
    SwapMode,
    Quit,
}

/// A page, and how it was left.
#[derive(Debug)]
struct Visit {
    /// What tells this visit apart from others to the same file.
    id: u64,
    opened: Opened,
    focus: Focus,
    /// Whether to bring what is focused into view.
    reveal: bool,
    focused_link: Option<usize>,
    /// Whether to bring the focused link into view.
    reveal_link: bool,
    /// The line of the heading or footnote to scroll to.
    go_to: Option<usize>,
    /// The offset to scroll the page to.
    scroll_to: Option<f32>,
    /// The line of a block to scroll sideways, and the offset to go to.
    nudge: Option<(usize, f32)>,
    view: View,
}

impl Visit {
    /// The line of the block that the focused table is in.
    fn focused_block(&self) -> Option<usize> {
        let table = self.focus.table(self.opened.tables())?;
        Some(table.line)
    }

    /// Focuses the link this many places along, wrapping around at either end.
    fn move_link(&mut self, next: impl FnOnce(Option<usize>, usize) -> usize) {
        let count = self.view.links.len();
        if count > 0 {
            self.focused_link = Some(next(self.focused_link, count));
            self.reveal_link = true;
        }
    }
}

/// What is showing around and over the page.
#[derive(Debug, Clone, Copy)]
struct Panels {
    tree: bool,
    tests: bool,
    help: bool,
}

/// The graphical interface.
#[derive(Debug)]
pub struct Gui {
    session: Session,
    pages: History<Visit>,
    tree: FileTree,
    /// Every test file of the workspace, for marking them in the tree.
    test_files: BTreeMap<String, Opened>,
    panels: Panels,
    search: Option<Search>,
    /// What is typed in the search box, as egui edits it.
    query: String,
    /// Whether search shows only test files. Outlives the search box.
    only_tests: bool,
    status: Option<String>,
    /// What stopped the last run, if anything did.
    failure: Option<String>,
    /// Where the link under the pointer goes.
    hovered_link: Option<String>,
    /// The last address opened in the web browser.
    opened_url: Option<String>,
    /// The text last put on the clipboard.
    copied: Option<String>,
    selected: Option<String>,
    pictures: Pictures,
    /// How many visits there have been.
    visits: u64,
    commands: Vec<Command>,
    /// Whether the fonts and colours are in place.
    installed: bool,
    open: bool,
}

/// Opens the window, and returns once it closes.
///
/// # Errors
///
/// Fails with a message for the user when temoc-ui cannot start, or when the
/// window cannot open.
pub fn open(launch: &Launch) -> Result<(), String> {
    let gui = Gui::start(launch)?;
    let viewport = egui::ViewportBuilder::default()
        .with_title("temoc-ui")
        .with_inner_size(WINDOW);
    let options = eframe::NativeOptions {
        viewport,
        ..eframe::NativeOptions::default()
    };
    eframe::run_native("temoc-ui", options, Box::new(|_| Ok(Box::new(gui))))
        .map_err(|error| error.to_string())
}

impl eframe::App for Gui {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}

impl Gui {
    /// Finds the workspace and opens what opens first, before any window shows.
    ///
    /// # Errors
    ///
    /// Fails with a message for the user when the arguments make no sense or
    /// the path given cannot be opened.
    pub fn start(launch: &Launch) -> Result<Self, String> {
        let arguments = Arguments::read(&launch.arguments)?;
        if arguments.theme.is_some() {
            return Err("--theme does not work with --gui".into());
        }
        let session = Session::start(&Start {
            current_dir: launch.current_dir.clone(),
            path: arguments.path.map(String::from),
            config: arguments.config.map(String::from),
            extra_path: launch.extra_path.clone(),
        })?;
        let first_file = session.first_file().map(String::from);
        let tree = FileTree::open(session.workspace(), first_file.as_deref());
        let test_files = workspace::markdown_files(session.workspace())
            .into_iter()
            .filter_map(|file| match session.read(&file) {
                Ok(opened) if opened.is_test_file() => Some((file, opened)),
                // Opening a file that cannot be read says what is wrong with it.
                Ok(_) | Err(_) => None,
            })
            .collect();
        let mut gui = Self {
            session,
            pages: History::default(),
            tree,
            test_files,
            panels: Panels {
                tree: true,
                tests: true,
                help: false,
            },
            search: None,
            query: String::new(),
            only_tests: false,
            status: None,
            failure: None,
            hovered_link: None,
            opened_url: None,
            copied: None,
            selected: None,
            pictures: Pictures::default(),
            visits: 0,
            commands: Vec::new(),
            installed: false,
            open: true,
        };
        match first_file {
            Some(file) => gui.visit(&file, None)?,
            None => gui.open_search(),
        }
        Ok(gui)
    }

    /// Whether temoc is running.
    #[must_use]
    pub fn is_busy(&self) -> bool {
        self.session.is_busy()
    }

    /// The arguments of the last temoc command.
    #[must_use]
    pub fn temoc_arguments(&self) -> Option<&[String]> {
        self.session.temoc_arguments()
    }

    /// The last address opened in the web browser.
    #[must_use]
    pub fn opened_url(&self) -> Option<&str> {
        self.opened_url.as_deref()
    }

    /// The text last put on the clipboard.
    #[must_use]
    pub fn copied(&self) -> Option<&str> {
        self.copied.as_deref()
    }

    /// The text selected on the page.
    #[must_use]
    pub fn selected(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    /// Where the page is scrolled to.
    #[must_use]
    pub fn page_position(&self) -> Option<Position> {
        self.pages.current().map(|visit| visit.view.position())
    }

    /// Where the block at this height of the window is scrolled to.
    #[must_use]
    pub fn sideways_at(&self, height: f32) -> Option<Position> {
        let visit = self.pages.current()?;
        visit.view.sideways_at(height)
    }

    /// Where the page shows in the window.
    #[must_use]
    pub fn page_rect(&self) -> Option<egui::Rect> {
        self.pages.current().map(|visit| visit.view.rect)
    }

    /// The part of this rectangle of the window that shows on the page, if
    /// any of it does.
    #[must_use]
    pub fn visible_part(&self, rect: egui::Rect) -> Option<egui::Rect> {
        self.pages.current()?.view.visible_part(rect)
    }

    /// `false` once the window has been asked to close.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Draws one frame, and reacts to what happened since the last one.
    pub fn show(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        if !self.installed {
            look::install(&ctx);
            self.installed = true;
        }
        if ctx.input(|input| input.viewport().close_requested()) {
            self.open = false;
        }
        if let Some(failure) = self.session.update() {
            self.status = Some(failure.clone());
            self.failure = Some(failure);
        }
        let (selected, copied) = selection::read(&ctx);
        self.selected = selected;
        if let Some(copied) = copied {
            self.status = Some(clipboard::copied(&copied));
            self.copied = Some(copied);
        }
        self.read_keys(&ctx);
        let palette = look::palette(ctx.theme());
        self.toolbar(ui, palette);
        self.status_bar(ui, palette);
        if self.panels.tree {
            self.tree_panel(ui, palette);
        }
        if self.panels.tests && self.opened().is_some_and(Opened::is_test_file) {
            self.tests_panel(ui, palette);
        }
        self.page(ui, palette);
        if self.search.is_some() {
            self.search_box(&ctx);
        }
        if self.panels.help {
            help(&ctx);
        }
        self.carry_out(&ctx);
        if self.session.is_busy() {
            ctx.request_repaint_after(TICK);
        }
    }

    fn opened(&self) -> Option<&Opened> {
        self.pages.current().map(|visit| &visit.opened)
    }

    /// Goes to a file of the workspace, and to a heading of it if there is one,
    /// leaving the current page behind.
    fn visit(&mut self, file: &str, anchor: Option<&str>) -> Result<(), String> {
        let opened = self.session.read(file)?;
        let headings = opened.document().headings();
        let mut headings = headings.iter();
        let heading = anchor.and_then(|anchor| headings.find(|known| known.anchor == anchor));
        self.visits = self.visits.saturating_add(1);
        self.pages.visit(Visit {
            id: self.visits,
            focus: Focus::first_of(opened.tables()),
            go_to: heading.map(|heading| heading.line),
            opened,
            reveal: false,
            focused_link: None,
            reveal_link: false,
            scroll_to: None,
            nudge: None,
            view: View::default(),
        });
        self.tree.reveal(file);
        Ok(())
    }

    fn in_visit(&mut self, change: impl FnOnce(&mut Visit)) {
        if let Some(visit) = self.pages.current_mut() {
            change(visit);
        }
    }

    /// Changes what is focused on the page, and brings it into view.
    fn focus(&mut self, change: impl FnOnce(&mut Focus, &[TestTable])) {
        if let Some(visit) = self.pages.current_mut() {
            change(&mut visit.focus, visit.opened.tables());
            visit.reveal = true;
        }
    }

    /// Starts what a command asks to run.
    fn start_run(&mut self, command: &Command) -> Result<(), String> {
        self.status = None;
        self.failure = None;
        let visit = self.pages.current();
        let session = &mut self.session;
        match command {
            Command::RunRow(line) => session.run_row(visit.map(|visit| (&visit.opened, *line))),
            Command::RunFocused => {
                let focus = visit.map(|visit| (visit, visit.opened.tables()));
                let row = focus
                    .and_then(|(visit, tables)| Some((&visit.opened, visit.focus.row(tables)?)));
                if row.is_some() {
                    return session.run_row(row);
                }
                session.run_table(focus.and_then(|(visit, tables)| {
                    let table = visit.focus.table(tables)?;
                    Some((&visit.opened, table.directive_line))
                }))
            }
            _ => session.run_file(visit.map(|visit| &visit.opened)),
        }
    }

    fn open_search(&mut self) {
        self.query.clear();
        self.search = Some(Search::open(self.session.workspace(), self.only_tests));
    }

    /// Takes the keys meant for temoc-ui out of this frame's input.
    fn read_keys(&mut self, ctx: &Context) {
        let searching = self.search.is_some();
        let help_open = self.panels.help;
        let mut commands = Vec::new();
        ctx.input_mut(|input| {
            input.events.retain(|event| {
                let command = match event {
                    Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } => key_command(*key, *modifiers, searching, help_open),
                    Event::PointerButton {
                        button,
                        pressed: true,
                        ..
                    } => button_command(*button),
                    Event::Text(text) if !searching => text_command(text),
                    _ => None,
                };
                let taken = command.is_some();
                commands.extend(command);
                !taken
            });
        });
        self.commands.extend(commands);
    }

    fn carry_out(&mut self, ctx: &Context) {
        for command in std::mem::take(&mut self.commands) {
            let result = self.apply(ctx, command);
            if let Err(message) = result {
                self.status = Some(message);
            }
        }
    }

    fn apply(&mut self, ctx: &Context, command: Command) -> Result<(), String> {
        match command {
            Command::Open(file) => {
                self.status = None;
                self.visit(&file, None)?;
            }
            Command::Follow(target) => self.follow(ctx, &target)?,
            Command::FollowFocused => {
                let visit = self.pages.current();
                let link = visit.and_then(|visit| visit.view.links.get(visit.focused_link?));
                if let Some(link) = link.cloned() {
                    self.apply(ctx, link)?;
                }
            }
            Command::GoToLine(line) => self.in_visit(|visit| visit.go_to = Some(line)),
            Command::Copy(text) => ctx.copy_text(text),
            Command::NextLink => self.in_visit(|visit| {
                visit.move_link(|focused, count| match focused {
                    Some(index) => index.saturating_add(1) % count,
                    None => 0,
                });
            }),
            Command::PreviousLink => self.in_visit(|visit| {
                visit.move_link(|focused, count| {
                    let previous = focused.and_then(|index| index.checked_sub(1));
                    previous.unwrap_or(count.saturating_sub(1))
                });
            }),
            Command::Scroll(scroll) => self.in_visit(|visit| {
                visit.scroll_to = Some(visit.view.scrolled(scroll));
            }),
            Command::Sideways(way) => {
                let pointer = ctx.pointer_latest_pos();
                self.in_visit(|visit| {
                    let fallback = visit.focused_block();
                    visit.nudge = visit.view.nudged(pointer, fallback, way);
                });
            }
            Command::ToggleFolder(row) => {
                if self.tree.select_shown(row) {
                    self.tree.toggle_folder();
                }
            }
            Command::Back => {
                self.pages.back();
                self.reveal_document();
            }
            Command::Forward => {
                self.pages.forward();
                self.reveal_document();
            }
            Command::OpenSearch => self.open_search(),
            Command::CloseSearch => self.search = None,
            Command::OpenResult(index) => self.open_result(index)?,
            Command::OnlyTestFiles => {
                self.only_tests = !self.only_tests;
                if let Some(search) = &mut self.search {
                    search.toggle_only_tests();
                }
            }
            Command::NextTable => self.focus(Focus::next_table),
            Command::PreviousTable => self.focus(|focus, _| focus.previous_table()),
            Command::NextRow => self.focus(Focus::next_row),
            Command::PreviousRow => self.focus(Focus::previous_row),
            Command::Focus { table, line } => self.focus(|focus, tables| {
                focus.focus_table(table);
                focus.focus_line(tables, line);
            }),
            Command::RunFile | Command::RunFocused | Command::RunRow(_) => {
                self.start_run(&command)?;
            }
            Command::ToggleTree => self.panels.tree = !self.panels.tree,
            Command::ToggleTests => self.panels.tests = !self.panels.tests,
            Command::ToggleHelp => self.panels.help = !self.panels.help,
            Command::CloseHelp => self.panels.help = false,
            Command::SwapMode => ctx.set_theme(match ctx.theme() {
                Theme::Dark => Theme::Light,
                Theme::Light => Theme::Dark,
            }),
            Command::Quit => {
                self.open = false;
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
        }
        Ok(())
    }

    fn reveal_document(&mut self) {
        if let Some(file) = self.opened().map(|opened| opened.file().to_owned()) {
            self.tree.reveal(&file);
        }
    }

    /// Follows a link of the open document.
    fn follow(&mut self, ctx: &Context, target: &str) -> Result<(), String> {
        if is_website(target) {
            ctx.open_url(OpenUrl::new_tab(target));
            self.opened_url = Some(target.to_owned());
            self.status = Some(format!("Opened in your browser: {target}"));
            return Ok(());
        }
        let Some(document) = self.opened().map(|opened| opened.file().to_owned()) else {
            return Ok(());
        };
        let place = self.session.follow(&document, target)?;
        self.status = None;
        self.visit(&place.file, place.anchor.as_deref())
    }

    /// Opens a result of the search, or the chosen one.
    fn open_result(&mut self, index: usize) -> Result<(), String> {
        let hits = self.search.as_ref().map(Search::results);
        let Some(hit) = hits.and_then(|hits| hits.get(index)).cloned() else {
            return Ok(());
        };
        self.search = None;
        self.status = None;
        let anchor = hit.heading.map(|heading| heading.anchor);
        self.visit(&hit.file, anchor.as_deref())
    }

    fn toolbar(&mut self, ui: &mut Ui, palette: &Palette) {
        let document = self.opened().map(|opened| opened.file().to_owned());
        let test_page = self.opened().is_some_and(Opened::is_test_file);
        let busy = self.session.is_busy();
        let mut commands = Vec::new();
        Panel::top("toolbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                let tree = (Icon::LeftPanel, "Show or hide the file tree");
                icon_button(ui, tree, Command::ToggleTree, &mut commands);
                icon_button(ui, (Icon::Back, "Go back"), Command::Back, &mut commands);
                let forward = (Icon::Forward, "Go forward");
                icon_button(ui, forward, Command::Forward, &mut commands);
                if let Some(document) = &document {
                    let response = ui.label(document);
                    tagged(&response, tags::DOCUMENT, document);
                }
                let search = ui.add(Button::new("Search files and headings"));
                if search.clicked() {
                    commands.push(Command::OpenSearch);
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if test_page {
                        let run = ui.add_enabled_ui(!busy, |ui| {
                            icons::run_icon(ui, "Run file", palette.passed, icons::LARGE_RUN)
                        });
                        if run.inner.clicked() {
                            commands.push(Command::RunFile);
                        }
                    }
                    let tests = (Icon::RightPanel, "Show or hide the tests panel");
                    icon_button(ui, tests, Command::ToggleTests, &mut commands);
                    let help = (Icon::Help, "Show help");
                    icon_button(ui, help, Command::ToggleHelp, &mut commands);
                    let mode = (Icon::Contrast, "Swap between dark and light");
                    icon_button(ui, mode, Command::SwapMode, &mut commands);
                });
            });
        });
        self.commands.extend(commands);
    }

    fn status_bar(&self, ui: &mut Ui, palette: &Palette) {
        let running = self.session.running_file();
        let running = running.map(|file| format!("Running {file}..."));
        let status = self.status.clone().or(running);
        let result = self
            .opened()
            .and_then(|opened| self.session.file_result(opened));
        Panel::bottom("status").show(ui, |ui| {
            ui.horizontal(|ui| {
                if let Some(status) = status.or_else(|| self.hovered_link.clone()) {
                    let response = ui.label(&status);
                    tagged(&response, tags::STATUS, &status);
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if let Some(result) = result {
                        let text = RichText::new(format!("file: {result}"))
                            .color(result_colour(result, palette));
                        let response = ui.label(text);
                        tagged(&response, tags::FILE_RESULT, result);
                    }
                });
            });
        });
    }

    fn tree_panel(&mut self, ui: &mut Ui, palette: &Palette) {
        let document = self.opened().map(|opened| opened.file().to_owned());
        let mut commands = Vec::new();
        Panel::left("tree").default_size(248.0).show(ui, |ui| {
            let heading = ui.label(RichText::new("Files").color(palette.line));
            tagged(&heading, tags::TREE, "Files");
            ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                for (index, row) in self.tree.rows().iter().enumerate() {
                    let selected = Some(row.path()) == document.as_deref();
                    let indent = "  ".repeat(row.depth);
                    let opened = self.test_files.get(row.path());
                    let mark = opened
                        .and_then(|opened| self.session.file_result(opened))
                        .and_then(mark_of);
                    let text = format!("{indent}{}", row.label());
                    let button = Button::selectable(selected, text)
                        .right_text(mark.unwrap_or_default())
                        .min_size(vec2(ui.available_width(), 0.0));
                    let response = ui.add(button);
                    let name = if row.is_folder() {
                        format!("{}/", row.name)
                    } else {
                        row.name.clone()
                    };
                    response.widget_info(|| {
                        WidgetInfo::selected(Role::TreeItem, true, selected, &name)
                    });
                    ui.ctx().accesskit_node_builder(response.id, |node| {
                        node.set_selected(selected);
                        if opened.is_some() {
                            node.set_description(tags::TEST_FILE);
                        }
                        if let Some(mark) = mark {
                            node.set_value(mark);
                        }
                    });
                    if response.clicked() {
                        commands.push(if row.is_folder() {
                            Command::ToggleFolder(index)
                        } else {
                            Command::Open(row.path().to_owned())
                        });
                    }
                }
            });
        });
        self.commands.extend(commands);
    }

    fn tests_panel(&mut self, ui: &mut Ui, palette: &Palette) {
        let Some(visit) = self.pages.current() else {
            return;
        };
        let opened = &visit.opened;
        let tables = opened.tables();
        let outcome = self.session.outcome(opened);
        let results = self.session.results(opened.file());
        let failures = results.map(|results| results.not_passed(tables));
        let failures = failures.unwrap_or_default();
        let command = self.session.temoc_arguments();
        let command = command.map(|arguments| format!("temoc {}", arguments.join(" ")));
        let mut commands = Vec::new();
        Panel::right("tests").default_size(284.0).show(ui, |ui| {
            let heading = ui.label(RichText::new("This file").color(palette.line));
            tagged(&heading, tags::TESTS_PANEL, "This file");
            if let Some(result) = outcome.file {
                let text = RichText::new(result).color(result_colour(result, palette));
                ui.label(text.size(17.0));
            }
            ui.add_space(12.0);
            ui.label(RichText::new("Tables").color(palette.line));
            for (index, (table, result)) in outcome.tables.iter().enumerate() {
                let focused = visit.focus.table_index() == Some(index);
                if panel_table(ui, table, result, focused) {
                    let table = index;
                    commands.push(Command::Focus { table, line: None });
                }
            }
            if !failures.is_empty() {
                ui.add_space(12.0);
                ui.label(RichText::new("What did not pass").color(palette.line));
                for (table, failure) in &failures {
                    if panel_failure(ui, table, failure, palette) {
                        let found = tables.iter().position(|known| known == *table);
                        let line = Some(failure.line);
                        commands.extend(found.map(|table| Command::Focus { table, line }));
                    }
                }
            }
            if let Some(command) = &command {
                ui.add_space(12.0);
                ui.label(RichText::new("Last command").color(palette.line));
                let text = RichText::new(command).monospace().color(palette.code);
                let response = ui.label(text);
                tagged(&response, tags::LAST_COMMAND, command);
            }
        });
        self.commands.extend(commands);
    }

    fn page(&mut self, ui: &mut Ui, palette: &Palette) {
        let banner = self.banner();
        let busy = self.session.is_busy();
        let mut wanted = markdown::Wanted::default();
        let mut reveal = false;
        let mut scroll_to = None;
        if let Some(visit) = self.pages.current_mut() {
            reveal = std::mem::take(&mut visit.reveal);
            scroll_to = visit.scroll_to.take();
            wanted = markdown::Wanted {
                visit: visit.id,
                focused_link: visit.focused_link,
                reveal_link: std::mem::take(&mut visit.reveal_link),
                go_to: visit.go_to.take(),
                nudge: visit.nudge.take(),
            };
        }
        let mut happened = markdown::Happened::default();
        let mut reach = None;
        let frame = Frame::new().fill(palette.page).inner_margin(PAGE_MARGIN);
        CentralPanel::default().frame(frame).show(ui, |ui| {
            if let Some(banner) = &banner {
                let frame = Frame::new().fill(palette.chip).inner_margin(10);
                frame.show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    let text = RichText::new(banner).color(palette.failed);
                    let response = ui.add(Label::new(text).wrap());
                    tagged(&response, tags::BANNER, banner);
                });
                ui.add_space(12.0);
            }
            let Some(visit) = self.pages.current() else {
                return;
            };
            let opened = &visit.opened;
            let page = markdown::Page {
                document: opened.document(),
                file: opened.file(),
                workspace: self.session.workspace(),
                tests: markdown::Tests {
                    tables: opened.tables(),
                    results: self.session.results(opened.file()),
                    focus: visit.focus,
                    busy,
                    reveal,
                },
                wanted,
            };
            let pictures = &mut self.pictures;
            let mut area = ScrollArea::vertical()
                .id_salt(("page", visit.id))
                .auto_shrink(false);
            if let Some(offset) = scroll_to {
                area = area.vertical_scroll_offset(offset);
            }
            let shown = area.show(ui, |ui| {
                happened = markdown::show(ui, page, pictures, palette);
            });
            let rect = shown.inner_rect;
            let size = shown.content_size.y;
            reach = Some((
                rect,
                Reach::of(shown.state.clamped_offset().y, size, rect.height()),
            ));
        });
        self.commands.extend(happened.commands);
        if self.hovered_link != happened.hovered_link {
            // The status bar is drawn before the page, so it needs a new frame.
            self.hovered_link = happened.hovered_link;
            ui.ctx().request_repaint();
        }
        if let (Some(visit), Some((rect, reach))) = (self.pages.current_mut(), reach) {
            visit.view = View {
                rect,
                reach,
                blocks: happened.blocks,
                links: happened.links,
            };
        }
    }

    /// What the banner says: why this page cannot run, or what stopped the run.
    fn banner(&self) -> Option<String> {
        let test_page = self.opened().is_some_and(Opened::is_test_file);
        let cannot_run = test_page.then(|| self.session.configuration().err());
        self.failure.clone().or(cannot_run.flatten())
    }

    fn search_box(&mut self, ctx: &Context) {
        let mut commands = Vec::new();
        let only_tests = self.only_tests;
        let width = SEARCH_WIDTH.min(ctx.content_rect().width() - 32.0);
        let left = (ctx.content_rect().width() - width) / 2.0;
        let query = &mut self.query;
        let Some(search) = &mut self.search else {
            return;
        };
        Area::new(Id::unique("search"))
            .order(Order::Foreground)
            .fixed_pos(egui::pos2(left, 88.0))
            .show(ctx, |ui| {
                Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_width(width);
                    let edit = TextEdit::singleline(query).desired_width(f32::INFINITY);
                    let response = ui.add(edit);
                    response.request_focus();
                    tagged(&response, tags::SEARCH, "Search");
                    if response.changed() {
                        follow_query(search, query);
                    }
                    let filter = ui.selectable_label(only_tests, "Test files only");
                    if filter.clicked() {
                        commands.push(Command::OnlyTestFiles);
                    }
                    ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                        for (index, hit) in search.results().iter().enumerate() {
                            let text = hit.to_string();
                            let chosen = index == search.selected();
                            let response = ui.add(Button::selectable(chosen, &text));
                            tagged(&response, tags::SEARCH_RESULT, &text);
                            if response.clicked() {
                                commands.push(Command::OpenResult(index));
                            }
                        }
                    });
                });
            });
        ctx.input_mut(|input| {
            if input.consume_key(Modifiers::NONE, Key::ArrowDown) {
                search.choose_next();
            }
            if input.consume_key(Modifiers::NONE, Key::ArrowUp) {
                search.choose_previous();
            }
            if input.consume_key(Modifiers::NONE, Key::Enter) {
                commands.push(Command::OpenResult(search.selected()));
            }
        });
        self.commands.extend(commands);
    }
}

/// Makes the search match what the box says, one character at a time.
fn follow_query(search: &mut Search, query: &str) {
    while !query.starts_with(search.query()) {
        search.delete_character();
    }
    let typed = query.get(search.query().len()..).unwrap_or_default();
    typed.chars().for_each(|c| search.type_character(c));
}

/// What a key does, for the keys that are not text.
fn key_command(
    key: Key,
    modifiers: Modifiers,
    searching: bool,
    help_open: bool,
) -> Option<Command> {
    match key {
        Key::Q if modifiers.command => return Some(Command::Quit),
        Key::F1 => return Some(Command::ToggleHelp),
        Key::Escape if searching => return Some(Command::CloseSearch),
        Key::Escape if help_open => return Some(Command::CloseHelp),
        _ if searching => return None,
        _ => {}
    }
    if modifiers.command {
        command_key(key)
    } else if modifiers.alt {
        alt_key(key, modifiers.shift)
    } else {
        plain_key(key, modifiers.shift)
    }
}

/// What a key does while `Ctrl` is held, which is `Cmd` on macOS.
fn command_key(key: Key) -> Option<Command> {
    match key {
        Key::B => Some(Command::ToggleTree),
        Key::K => Some(Command::OpenSearch),
        Key::R => Some(Command::RunFile),
        Key::Enter => Some(Command::RunFocused),
        Key::OpenBracket => Some(Command::Back),
        Key::CloseBracket => Some(Command::Forward),
        _ => None,
    }
}

/// What a key does while `Alt` is held.
fn alt_key(key: Key, shift: bool) -> Option<Command> {
    match key {
        Key::ArrowLeft => Some(Command::Back),
        Key::ArrowRight => Some(Command::Forward),
        Key::ArrowDown if shift => Some(Command::NextRow),
        Key::ArrowUp if shift => Some(Command::PreviousRow),
        Key::ArrowDown => Some(Command::NextTable),
        Key::ArrowUp => Some(Command::PreviousTable),
        _ => None,
    }
}

/// What a key does by itself, or with `Shift` where that makes a difference.
fn plain_key(key: Key, shift: bool) -> Option<Command> {
    match key {
        Key::Tab if shift => Some(Command::PreviousLink),
        Key::Tab => Some(Command::NextLink),
        Key::Enter => Some(Command::FollowFocused),
        Key::ArrowDown => Some(Command::Scroll(Scroll::LineDown)),
        Key::ArrowUp => Some(Command::Scroll(Scroll::LineUp)),
        Key::PageDown | Key::Space => Some(Command::Scroll(Scroll::PageDown)),
        Key::PageUp => Some(Command::Scroll(Scroll::PageUp)),
        Key::Home => Some(Command::Scroll(Scroll::Top)),
        Key::End => Some(Command::Scroll(Scroll::Bottom)),
        Key::ArrowLeft => Some(Command::Sideways(Sideways::Left)),
        Key::ArrowRight => Some(Command::Sideways(Sideways::Right)),
        _ => None,
    }
}

/// What typing a character does, outside the search box.
fn text_command(text: &str) -> Option<Command> {
    match text {
        "/" => Some(Command::OpenSearch),
        "?" => Some(Command::ToggleHelp),
        _ => None,
    }
}

/// What a button of the mouse does, for the ones that are not for clicking.
fn button_command(button: PointerButton) -> Option<Command> {
    match button {
        PointerButton::Extra1 => Some(Command::Back),
        PointerButton::Extra2 => Some(Command::Forward),
        _ => None,
    }
}

/// Whether a link points away from the workspace.
fn is_website(target: &str) -> bool {
    target.starts_with("http://") || target.starts_with("https://")
}

/// The mark for a result written as a word.
fn mark_of(result: &str) -> Option<&'static str> {
    RESULTS
        .iter()
        .find(|known| known.name() == result)
        .map(|known| known.mark())
}

const RESULTS: [RunResult; 7] = [
    RunResult::Passed,
    RunResult::Skipped,
    RunResult::Snoozed,
    RunResult::Partial,
    RunResult::Failed,
    RunResult::Error,
    RunResult::Running,
];

fn result_colour(result: &str, palette: &Palette) -> Color32 {
    match result {
        "passed" => palette.passed,
        "snoozed" => palette.snoozed,
        "failed" | "error" => palette.failed,
        _ => palette.quote,
    }
}

/// Shows the keys over everything else.
fn help(ctx: &Context) {
    let width = 640.0_f32.min(ctx.content_rect().width() - 32.0);
    let left = (ctx.content_rect().width() - width) / 2.0;
    Area::new(Id::unique("help"))
        .order(Order::Foreground)
        .fixed_pos(egui::pos2(left, 48.0))
        .show(ctx, |ui| {
            Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_width(width);
                let heading = ui.heading("Keys");
                tagged(&heading, tags::HELP, "Keys");
                ui.add_space(8.0);
                egui::Grid::new("keys")
                    .spacing(Vec2::new(24.0, 4.0))
                    .show(ui, |ui| {
                        for (key, action) in KEYS {
                            ui.label(RichText::new(key).monospace());
                            ui.label(action);
                            ui.end_row();
                        }
                    });
            });
        });
}

/// A table in the tests panel. Says whether it was clicked.
fn panel_table(ui: &mut Ui, table: &str, result: &str, focused: bool) -> bool {
    let text = match mark_of(result) {
        Some(mark) => format!("{mark} {table}"),
        None => format!("  {table}"),
    };
    let button = Button::selectable(focused, text)
        .right_text(result)
        .min_size(vec2(ui.available_width(), 0.0));
    let response = ui.add(button);
    named(&response, Role::Button, table);
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_description(tags::PANEL_TABLE);
        node.set_value(result);
    });
    response.clicked()
}

/// A failure in the tests panel. Says whether it was clicked.
fn panel_failure(ui: &mut Ui, table: &TestTable, failure: &Failure, palette: &Palette) -> bool {
    let builder = UiBuilder::new()
        .id_salt(("failure", failure.line, failure.note()))
        .sense(Sense::click());
    let drawn = ui.scope_builder(builder, |ui| {
        ui.style_mut().interaction.selectable_labels = false;
        ui.set_width(ui.available_width());
        ui.label(format!("{}, line {}", table.name, failure.line));
        let note = RichText::new(failure.note()).size(12.5).color(palette.note);
        ui.add(Label::new(note).wrap());
    });
    let response = drawn.response;
    tagged(
        &response,
        tags::NOT_PASSED,
        &format!("{} {failure}", table.name),
    );
    response.clicked()
}

/// A button in the toolbar with an icon and the name it goes by.
fn icon_button(
    ui: &mut Ui,
    (icon, name): (Icon, &str),
    command: Command,
    commands: &mut Vec<Command>,
) {
    if icons::button(ui, icon, name).clicked() {
        commands.push(command);
    }
}

/// Gives a widget the name it goes by, whatever it shows.
fn named(response: &Response, role: Role, name: &str) {
    response.widget_info(|| WidgetInfo::labeled(role, response.enabled(), name));
}

/// Says what a widget is, for the accessibility tree.
fn tagged(response: &Response, tag: &str, name: &str) {
    named(response, Role::Label, name);
    response.ctx.accesskit_node_builder(response.id, |node| {
        node.set_description(tag);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_result_has_its_mark() {
        assert_eq!(mark_of("passed"), Some("✓"));
        assert_eq!(mark_of("failed"), Some("✗"));
        assert_eq!(mark_of("not run"), None);
    }

    #[test]
    fn websites_are_told_from_files() {
        assert!(is_website("https://github.com/killertux/temoc"));
        assert!(!is_website("guide/getting-started.md"));
    }

    #[test]
    fn the_search_follows_what_the_box_says() {
        let mut search = Search::open(std::path::Path::new("samples/docs"), false);
        follow_query(&mut search, "ref");
        assert_eq!(search.query(), "ref");
        follow_query(&mut search, "re");
        assert_eq!(search.query(), "re");
        follow_query(&mut search, "guide");
        assert_eq!(search.query(), "guide");
    }
}
