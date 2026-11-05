//! The `Gui` fixture: a fresh temoc-ui in a headless window, as described in
//! `spec/gui/writing-specs.md`.

use crate::keys::{Token, tokenize};
use crate::session::{NOT_IMPLEMENTED, example_binaries, samples};
use egui_kittest::kittest::{NodeT, Queryable};
use egui_kittest::{Harness, Node};
use std::thread::sleep;
use std::time::{Duration, Instant};
use temoc_ui::app::Launch;
use temoc_ui::gui::egui::accesskit::Role;
use temoc_ui::gui::egui::{
    Event, Key, Modifiers, MouseWheelSource, MouseWheelUnit, PointerButton, Pos2, Rect, Theme,
    ThemePreference, TouchPhase, Vec2, ViewportEvent, ViewportId,
};
use temoc_ui::gui::{Gui as Window, Position, tags};

const WIDTH: f32 = 1000.0;
const HEIGHT: f32 = 700.0;
const RUN_TIMEOUT: Duration = Duration::from_mins(1);
const POLL_INTERVAL: Duration = Duration::from_millis(10);
/// How far one step of a mouse wheel goes.
const WHEEL_STEP: f32 = 40.0;
/// How far one step sideways goes, which is a nudge on a trackpad.
const TRACKPAD_STEP: f32 = 10.0;

/// One of the actions described in `spec/gui/writing-specs.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Action {
    Press(Key, Modifiers),
    Character(char),
    Text(String),
    Click(String),
    /// The control with this name in the test table with that name.
    ClickIn(String, String),
    ClickRow(String),
    ClickTree(String),
    Hover(String),
    HoverRow(String),
    /// Turning the wheel over text, which moves the page by this much.
    Wheel(String, Vec2),
    MouseButton(PointerButton),
    Leave,
    WaitRow(String),
    CloseWindow,
}

impl Action {
    fn read_all(actions: &str) -> Result<Vec<Self>, String> {
        let mut tokens = tokenize(actions)?.into_iter().peekable();
        let mut read = Vec::new();
        while let Some(token) = tokens.next() {
            read.push(match token {
                Token::Quoted(text) => Self::Text(text),
                Token::Word(word) => match word.as_str() {
                    "Click" => match tokens.next() {
                        Some(Token::Quoted(name)) => {
                            if tokens.next_if(|token| is_word(token, "in")).is_some() {
                                Self::ClickIn(name, quoted(tokens.next(), "Click in")?)
                            } else {
                                Self::Click(name)
                            }
                        }
                        Some(Token::Word(word)) if word == "tree" => {
                            Self::ClickTree(quoted(tokens.next(), "Click tree")?)
                        }
                        Some(Token::Word(word)) if word == "row" => {
                            Self::ClickRow(line(tokens.next(), "Click row")?)
                        }
                        _ => return Err(NOT_IMPLEMENTED.into()),
                    },
                    "Hover" => match tokens.next() {
                        Some(Token::Quoted(text)) => Self::Hover(text),
                        Some(Token::Word(word)) if word == "row" => {
                            Self::HoverRow(line(tokens.next(), "Hover row")?)
                        }
                        _ => return Err(NOT_IMPLEMENTED.into()),
                    },
                    "Wheel" => match tokens.next() {
                        Some(Token::Quoted(text)) => Self::Wheel(text, Vec2::new(0.0, -WHEEL_STEP)),
                        Some(Token::Word(word)) if word == "sideways" => {
                            let text = quoted(tokens.next(), "Wheel sideways")?;
                            Self::Wheel(text, Vec2::new(-TRACKPAD_STEP, 0.0))
                        }
                        _ => return Err(format!("unknown action in `{actions}`")),
                    },
                    "Mouse" => match tokens.next() {
                        Some(Token::Word(word)) if word == "back" => {
                            Self::MouseButton(PointerButton::Extra1)
                        }
                        Some(Token::Word(word)) if word == "forward" => {
                            Self::MouseButton(PointerButton::Extra2)
                        }
                        _ => return Err(format!("unknown action in `{actions}`")),
                    },
                    "Leave" => Self::Leave,
                    "Wait" => match tokens.next() {
                        Some(Token::Word(word)) if word == "row" => {
                            Self::WaitRow(line(tokens.next(), "Wait row")?)
                        }
                        _ => return Err(format!("unknown action in `{actions}`")),
                    },
                    "Close" => match tokens.next() {
                        Some(Token::Word(word)) if word == "window" => Self::CloseWindow,
                        _ => return Err(format!("unknown action in `{actions}`")),
                    },
                    _ => key(&word).ok_or(NOT_IMPLEMENTED)?,
                },
            });
        }
        Ok(read)
    }
}

fn is_word(token: &Token, expected: &str) -> bool {
    matches!(token, Token::Word(word) if word == expected)
}

/// The line of a row, as the row goes by it.
fn line(token: Option<Token>, action: &str) -> Result<String, String> {
    match token {
        Some(Token::Word(line)) => Ok(line),
        _ => Err(format!("`{action}` needs a line")),
    }
}

fn quoted(token: Option<Token>, action: &str) -> Result<String, String> {
    match token {
        Some(Token::Quoted(text)) => Ok(text),
        _ => Err(format!("`{action}` needs quoted text")),
    }
}

/// A key as written in an action.
fn key(word: &str) -> Option<Action> {
    let mut rest = word;
    let mut modifiers = Modifiers::NONE;
    for (prefix, modifier) in [
        ("Ctrl-", Modifiers::COMMAND),
        ("Alt-", Modifiers::ALT),
        ("Shift-", Modifiers::SHIFT),
    ] {
        if let Some(after) = rest.strip_prefix(prefix) {
            rest = after;
            modifiers |= modifier;
        }
    }
    let mut characters = rest.chars();
    if let (Some(character), None) = (characters.next(), characters.next()) {
        if modifiers.is_none() {
            return Some(Action::Character(character));
        }
        let key = Key::from_name(&character.to_uppercase().to_string())?;
        return Some(Action::Press(key, modifiers));
    }
    let key = match rest {
        "Esc" => Key::Escape,
        "Up" => Key::ArrowUp,
        "Down" => Key::ArrowDown,
        "Left" => Key::ArrowLeft,
        "Right" => Key::ArrowRight,
        named => Key::from_name(named)?,
    };
    Some(Action::Press(key, modifiers))
}

/// What happened when a scenario ran.
pub enum Played {
    /// temoc-ui refused to open, with its message.
    Refused(String),
    /// The scenario could not be carried out.
    Broken(String),
    /// The scenario ran, and this is the window.
    Finished(Box<Scene>),
}

/// A temoc-ui in a headless window.
pub struct Scene {
    harness: Harness<'static, Window>,
    /// Whether actions wait for a run to finish. See `Wait row`.
    waits_for_runs: bool,
}

impl Played {
    /// Starts temoc-ui in a sample workspace and takes the actions.
    pub fn of(workspace: &str, arguments: &str, actions: &str, system_mode: &str) -> Self {
        let launch = Launch {
            current_dir: samples().join(workspace),
            arguments: arguments.split_whitespace().map(String::from).collect(),
            extra_path: example_binaries().into_iter().collect(),
            ..Launch::default()
        };
        let window = match Window::start(&launch) {
            Ok(window) => window,
            Err(message) => return Self::Refused(message),
        };
        let played = Action::read_all(actions).and_then(|actions| {
            let mut scene = Scene::open(window, system_mode)?;
            scene.play(&actions)?;
            Ok(scene)
        });
        match played {
            Ok(scene) => Self::Finished(Box::new(scene)),
            Err(problem) => Self::Broken(problem),
        }
    }
}

impl Scene {
    fn open(window: Window, system_mode: &str) -> Result<Self, String> {
        let system = match system_mode {
            "" => None,
            "dark" => Some(Theme::Dark),
            "light" => Some(Theme::Light),
            other => return Err(format!("unknown system mode: {other}")),
        };
        let mut harness = Harness::builder()
            .with_size(Vec2::new(WIDTH, HEIGHT))
            .build_ui_state(|ui, window: &mut Window| window.show(ui), window);
        // The harness picks a mode of its own, and the window follows the system.
        harness.ctx.set_theme(ThemePreference::System);
        harness.input_mut().system_theme = system;
        let mut scene = Self {
            harness,
            waits_for_runs: true,
        };
        scene.settle()?;
        Ok(scene)
    }

    fn play(&mut self, actions: &[Action]) -> Result<(), String> {
        for (index, action) in actions.iter().enumerate() {
            self.take(action)?;
            // The run that `Wait row` looks at may never finish by itself.
            if matches!(actions.get(index + 1), Some(Action::WaitRow(_))) {
                self.waits_for_runs = false;
            }
            self.settle()?;
        }
        Ok(())
    }

    fn take(&mut self, action: &Action) -> Result<(), String> {
        match action {
            Action::Press(key, modifiers) => {
                self.harness.key_press_modifiers(*modifiers, *key);
            }
            Action::Character(character) => self.type_character(*character),
            Action::Text(text) => text.chars().for_each(|c| self.type_character(c)),
            Action::Click(name) => self.named(name)?.click(),
            Action::ClickIn(name, table) => {
                let mut buttons = self.harness.query_all_by_role(Role::Button);
                let button = buttons.find(|button| {
                    let description = button.accesskit_node().description();
                    label_of(button) == *name && description.as_deref() == Some(table)
                });
                button.ok_or(format!("no `{name}` in `{table}`"))?.click();
            }
            Action::ClickRow(line) => self.row(line)?.click(),
            Action::ClickTree(name) => {
                let folder = format!("{name}/");
                let mut rows = self.harness.query_all_by_role(Role::TreeItem);
                let row = rows.find(|row| {
                    let label = label_of(row);
                    label == *name || label == folder
                });
                row.ok_or(format!("no `{name}` in the tree"))?.click();
            }
            Action::Hover(text) => self.named(text)?.hover(),
            Action::HoverRow(line) => self.row(line)?.hover(),
            Action::Wheel(text, delta) => {
                self.hover_over(text)?;
                // The pointer has to be there before the wheel turns.
                self.harness.run_ok();
                self.harness.event(Event::MouseWheel {
                    unit: MouseWheelUnit::Point,
                    delta: *delta,
                    phase: TouchPhase::Move,
                    source: MouseWheelSource::Wheel,
                    modifiers: Modifiers::NONE,
                });
            }
            Action::MouseButton(button) => {
                let pos = Pos2::new(WIDTH / 2.0, HEIGHT / 2.0);
                for pressed in [true, false] {
                    self.harness.event(Event::PointerButton {
                        pos,
                        button: *button,
                        pressed,
                        modifiers: Modifiers::NONE,
                    });
                }
            }
            Action::Leave => self.harness.hover_at(Pos2::new(WIDTH, HEIGHT)),
            Action::WaitRow(line) => self.wait_for_row(line)?,
            Action::CloseWindow => {
                let viewports = &mut self.harness.input_mut().viewports;
                let root = viewports.entry(ViewportId::ROOT).or_default();
                root.events.push(ViewportEvent::Close);
            }
        }
        Ok(())
    }

    /// A character typed on the keyboard, which is a key and then its text.
    fn type_character(&self, character: char) {
        if let Some(key) = Key::from_name(&character.to_uppercase().to_string()) {
            self.harness.key_press(key);
        }
        self.harness.event(Event::Text(character.to_string()));
    }

    /// The first thing with this name, which for text is what it says.
    fn named<'scene>(&'scene self, name: &'scene str) -> Result<Node<'scene>, String> {
        let mut found = self.harness.query_all_by(move |node| {
            node.label().as_deref() == Some(name) || node.value().as_deref() == Some(name)
        });
        found.next().ok_or(format!("nothing is called `{name}`"))
    }

    fn test_rows(&self) -> impl Iterator<Item = Node<'_>> {
        let tag = Some(tags::TEST_ROW);
        self.harness
            .query_all_by(move |node| node.description().as_deref() == tag)
    }

    fn row(&self, line: &str) -> Result<Node<'_>, String> {
        let mut rows = self.test_rows();
        let row = rows.find(|row| label_of(row) == line);
        row.ok_or(format!("no test row on line {line}"))
    }

    /// Lets the run go on until a row is running.
    fn wait_for_row(&mut self, line: &str) -> Result<(), String> {
        let deadline = Instant::now() + RUN_TIMEOUT;
        loop {
            self.harness.step();
            let row = self.row(line).ok();
            let result = row.and_then(|row| row.accesskit_node().value());
            if result.as_deref() == Some("running") {
                return Ok(());
            }
            if Instant::now() > deadline {
                return Err(format!("row {line} never started"));
            }
            sleep(POLL_INTERVAL);
        }
    }

    /// Lets the window react, and waits for any test run to finish.
    fn settle(&mut self) -> Result<(), String> {
        // An action only reaches the window here, and may start a run.
        self.harness.run_ok();
        let deadline = Instant::now() + RUN_TIMEOUT;
        while self.waits_for_runs && self.harness.state().is_busy() {
            if Instant::now() > deadline {
                return Err("the run did not finish".into());
            }
            self.harness.step();
            sleep(POLL_INTERVAL);
        }
        self.harness.run_ok();
        Ok(())
    }

    /// The window itself.
    pub fn window(&self) -> &Window {
        self.harness.state()
    }

    /// Which mode shows.
    pub fn mode(&self) -> Theme {
        self.harness.ctx.theme()
    }

    /// How the Run file button is, if it is there.
    pub fn run_file(&self) -> Option<&'static str> {
        let button = self
            .harness
            .query_by_role_and_label(Role::Button, "Run file");
        button.map(|button| {
            if button.accesskit_node().is_disabled() {
                "disabled"
            } else {
                "enabled"
            }
        })
    }

    /// Moves the pointer over the part of this text that shows.
    fn hover_over(&self, text: &str) -> Result<(), String> {
        let rects = self.text_rects(text);
        let mut parts = rects.iter();
        let part = parts.find_map(|rect| self.window().visible_part(*rect));
        let part = part.ok_or(format!("`{text}` does not show"))?;
        self.harness.hover_at(part.center());
        Ok(())
    }

    /// Where each piece of text with this in it is.
    fn text_rects(&self, text: &str) -> Vec<Rect> {
        let nodes = self.harness.query_all_by(|node| {
            let says = |said: Option<String>| said.is_some_and(|said| said.contains(text));
            says(node.label()) || says(node.value())
        });
        nodes.map(|node| node.rect()).collect()
    }

    /// Whether this text is on screen.
    pub fn shows(&self, text: &str) -> bool {
        let rects = self.text_rects(text);
        let mut rects = rects.iter();
        rects.any(|rect| self.window().visible_part(*rect).is_some())
    }

    /// Where the block holding this text is scrolled to.
    pub fn sideways(&self, text: &str) -> Option<Position> {
        let rects = self.text_rects(text);
        let mut positions = rects.iter();
        positions.find_map(|rect| self.window().sideways_at(rect.center().y))
    }

    /// Whether the mark of the row holding this text is on screen.
    pub fn shows_mark_of(&self, text: &str) -> bool {
        let rows = self.text_rects(text);
        let tag = Some(tags::MARK);
        let marks = self
            .harness
            .query_all_by(move |node| node.description().as_deref() == tag);
        let mut marks = marks.map(|mark| mark.rect());
        marks.any(|mark| {
            let beside = |row: &Rect| row.y_range().contains(mark.center().y);
            let page = self.window().page_rect();
            rows.iter().any(beside) && page.is_some_and(|page| page.contains_rect(mark))
        })
    }

    /// The text of the focused link.
    pub fn focused_link(&self) -> Option<String> {
        self.first_tagged(tags::FOCUSED_LINK)
    }

    /// The focused test table, by its name.
    pub fn focused_table(&self) -> Option<String> {
        let tag = Some(tags::TEST_TABLE);
        let mut headers = self
            .harness
            .query_all_by(move |node| node.description().as_deref() == tag);
        let header = headers.find(|header| header.accesskit_node().is_selected() == Some(true));
        header.map(|header| label_of(&header))
    }

    /// The focused test row, by its line.
    pub fn focused_row(&self) -> Option<String> {
        let mut rows = self.test_rows();
        let row = rows.find(|row| row.accesskit_node().is_selected() == Some(true));
        row.map(|row| label_of(&row))
    }

    /// Each test row that has a result, written `line: result`.
    pub fn row_results(&self) -> Vec<String> {
        let rows = self.test_rows();
        rows.filter_map(|row| {
            let result = row.accesskit_node().value()?;
            Some(format!("{}: {result}", label_of(&row)))
        })
        .collect()
    }

    /// The Run row buttons showing, written `line: enabled` or `line: disabled`.
    pub fn run_rows(&self) -> Vec<String> {
        let buttons = self.harness.query_all_by_role(Role::Button);
        let buttons = buttons.filter(|button| label_of(button) == "Run row");
        buttons
            .map(|button| {
                let node = button.accesskit_node();
                let line = node.description().unwrap_or_default();
                let state = if node.is_disabled() {
                    "disabled"
                } else {
                    "enabled"
                };
                format!("{line}: {state}")
            })
            .collect()
    }

    /// The tables listed in the tests panel, written `name: result`.
    pub fn panel_tables(&self) -> Vec<String> {
        let tag = Some(tags::PANEL_TABLE);
        let tables = self
            .harness
            .query_all_by(move |node| node.description().as_deref() == tag);
        tables
            .map(|table| {
                let result = table.accesskit_node().value().unwrap_or_default();
                format!(
                    "{}: {result}",
                    table.accesskit_node().label().unwrap_or_default()
                )
            })
            .collect()
    }

    /// The notes under a test row.
    pub fn notes(&self, line: &str) -> Vec<String> {
        let prefix = format!("{line}: ");
        let notes = self.tagged(tags::NOTE).into_iter();
        let notes = notes.filter_map(|note| note.strip_prefix(&prefix).map(String::from));
        notes.collect()
    }

    /// The labels of everything with this tag, in order.
    pub fn tagged(&self, tag: &'static str) -> Vec<String> {
        let found = self
            .harness
            .query_all_by(move |node| node.description().as_deref() == Some(tag));
        found.map(|node| label_of(&node)).collect()
    }

    pub fn first_tagged(&self, tag: &'static str) -> Option<String> {
        self.tagged(tag).into_iter().next()
    }

    pub fn is_tagged(&self, tag: &'static str) -> bool {
        self.first_tagged(tag).is_some()
    }

    /// The rows of the file tree that this picks something out of.
    fn tree_rows<T>(&self, pick: impl Fn(&Node<'_>) -> Option<T>) -> Vec<T> {
        let rows = self.harness.query_all_by_role(Role::TreeItem);
        rows.filter_map(|row| pick(&row)).collect()
    }

    /// The rows of the file tree, in order.
    pub fn tree(&self) -> Vec<String> {
        self.tree_rows(|row| Some(label_of(row)))
    }

    /// The row of the file tree that is selected.
    pub fn tree_selection(&self) -> Option<String> {
        let selected = self.tree_rows(|row| {
            let selected = row.accesskit_node().is_selected() == Some(true);
            selected.then(|| label_of(row))
        });
        selected.into_iter().next()
    }

    /// The rows of the file tree marked as test files.
    pub fn tree_test_files(&self) -> Vec<String> {
        self.tree_rows(|row| {
            let description = row.accesskit_node().description();
            (description.as_deref() == Some(tags::TEST_FILE)).then(|| label_of(row))
        })
    }

    /// Each row of the file tree with a mark.
    pub fn tree_marks(&self) -> Vec<String> {
        self.tree_rows(|row| {
            let mark = row.accesskit_node().value()?;
            Some(format!("{}: {mark}", label_of(row)))
        })
    }
}

/// The name of a node. egui keeps the text of a label as its value.
fn label_of(node: &Node<'_>) -> String {
    let node = node.accesskit_node();
    let label = node.label().filter(|label| !label.is_empty());
    label.or_else(|| node.value()).unwrap_or_default()
}
