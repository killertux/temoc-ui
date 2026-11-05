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
    Event, Key, Modifiers, Pos2, Theme, ThemePreference, Vec2, ViewportEvent, ViewportId,
};
use temoc_ui::gui::{Gui as Window, tags};

const WIDTH: f32 = 1000.0;
const HEIGHT: f32 = 700.0;
const RUN_TIMEOUT: Duration = Duration::from_mins(1);
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// One of the actions described in `spec/gui/writing-specs.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Action {
    Press(Key, Modifiers),
    Character(char),
    Text(String),
    Click(String),
    ClickTree(String),
    Hover(String),
    Leave,
    CloseWindow,
}

impl Action {
    fn read_all(actions: &str) -> Result<Vec<Self>, String> {
        let mut tokens = tokenize(actions)?.into_iter();
        let mut read = Vec::new();
        while let Some(token) = tokens.next() {
            read.push(match token {
                Token::Quoted(text) => Self::Text(text),
                Token::Word(word) => match word.as_str() {
                    "Click" => match tokens.next() {
                        Some(Token::Quoted(name)) => Self::Click(name),
                        Some(Token::Word(word)) if word == "tree" => {
                            Self::ClickTree(quoted(tokens.next(), "Click tree")?)
                        }
                        _ => return Err(NOT_IMPLEMENTED.into()),
                    },
                    "Hover" => match tokens.next() {
                        Some(Token::Quoted(text)) => Self::Hover(text),
                        _ => return Err(NOT_IMPLEMENTED.into()),
                    },
                    "Leave" => Self::Leave,
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

fn quoted(token: Option<Token>, action: &str) -> Result<String, String> {
    match token {
        Some(Token::Quoted(text)) => Ok(text),
        _ => Err(format!("`{action}` needs quoted text")),
    }
}

/// A key as written in an action.
fn key(word: &str) -> Option<Action> {
    if let Some(rest) = word.strip_prefix("Ctrl-") {
        let key = Key::from_name(&rest.to_uppercase())?;
        return Some(Action::Press(key, Modifiers::COMMAND));
    }
    let mut characters = word.chars();
    if let (Some(character), None) = (characters.next(), characters.next()) {
        return Some(Action::Character(character));
    }
    let key = match word {
        "Esc" => Key::Escape,
        "Up" => Key::ArrowUp,
        "Down" => Key::ArrowDown,
        "Left" => Key::ArrowLeft,
        "Right" => Key::ArrowRight,
        named => Key::from_name(named)?,
    };
    Some(Action::Press(key, Modifiers::NONE))
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
        let mut scene = Self { harness };
        scene.settle()?;
        Ok(scene)
    }

    fn play(&mut self, actions: &[Action]) -> Result<(), String> {
        for action in actions {
            self.take(action)?;
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
            Action::Leave => self.harness.hover_at(Pos2::new(WIDTH, HEIGHT)),
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

    fn named<'scene>(&'scene self, name: &'scene str) -> Result<Node<'scene>, String> {
        let found = self.harness.query_all_by_label(name).next();
        found.ok_or(format!("nothing is called `{name}`"))
    }

    /// Lets the window react, and waits for any test run to finish.
    fn settle(&mut self) -> Result<(), String> {
        // An action only reaches the window here, and may start a run.
        self.harness.run_ok();
        let deadline = Instant::now() + RUN_TIMEOUT;
        while self.harness.state().is_busy() {
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
