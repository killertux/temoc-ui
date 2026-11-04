//! Running one scenario: a fresh temoc-ui in an in-memory terminal.

use crate::clipboard;
use crate::keys::{self, Input};
use crate::screen::{self, Found};
use crossterm::event::{Event, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::env;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant};
use temoc_ui::app::{App, AppState, Launch, StartError};

const WIDTH: u16 = 80;
const HEIGHT: u16 = 24;
const RUN_TIMEOUT: Duration = Duration::from_mins(1);
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Reported for anything not built yet. No test expects this value, so a test
/// cannot pass by accident while the feature is missing.
pub const NOT_IMPLEMENTED: &str = "not implemented";

/// What happened when a scenario ran.
pub enum Outcome {
    /// The application has not been built yet.
    NotBuilt,
    /// temoc-ui refused to open, with its message.
    Refused(String),
    /// The scenario itself could not be carried out.
    Broken(String),
    /// The scenario ran, and this is the result.
    Finished(Box<Session>),
}

/// A temoc-ui running in an in-memory terminal.
pub struct Session {
    app: App,
    terminal: Terminal<TestBackend>,
    drawn: bool,
    output: Vec<u8>,
}

impl Outcome {
    /// Starts temoc-ui in a sample workspace and presses the keys.
    pub fn of(workspace: &str, arguments: &str, keys: &str) -> Self {
        let launch = Launch {
            current_dir: samples().join(workspace),
            arguments: arguments.split_whitespace().map(String::from).collect(),
            extra_path: example_binaries().into_iter().collect(),
        };
        match App::start(&launch) {
            Err(StartError::NotImplemented) => Self::NotBuilt,
            Err(StartError::Refused(message)) => Self::Refused(message),
            Ok(app) => match Session::play(app, keys) {
                Ok(session) => Self::Finished(Box::new(session)),
                Err(problem) => Self::Broken(problem),
            },
        }
    }
}

fn samples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../samples")
}

/// The folder holding this binary, and so `calculator-server` too.
fn example_binaries() -> Option<PathBuf> {
    env::current_exe().ok()?.parent().map(Path::to_path_buf)
}

impl Session {
    fn play(app: App, keys: &str) -> Result<Self, String> {
        let terminal = Terminal::new(TestBackend::new(WIDTH, HEIGHT)).map_err(|e| e.to_string())?;
        let mut session = Self {
            app,
            terminal,
            drawn: false,
            output: Vec::new(),
        };
        session.settle()?;
        for input in keys::parse(keys)? {
            session.apply(&input)?;
        }
        Ok(session)
    }

    fn apply(&mut self, input: &Input) -> Result<(), String> {
        match input {
            Input::Text(text) => {
                for c in text.chars() {
                    self.apply(&keys::character(c))?;
                }
                Ok(())
            }
            Input::Key(code, modifiers) => self.send(&Event::Key(KeyEvent::new(*code, *modifiers))),
            Input::Wheel(kind, modifiers) => self.send(&mouse(*kind, (0, 0), *modifiers)),
            Input::ClickOutside => self.click((0, 0)),
            Input::Click(text) => {
                let found = self.locate(text)?;
                self.click(found.first)
            }
            Input::Drag { from, to } => {
                let start = self.locate(from)?.first;
                let end = self.locate(to)?.last;
                self.drag(start, end)
            }
            Input::DragBack { from, to } => {
                let start = self.locate(from)?.last;
                let end = self.locate(to)?.first;
                self.drag(start, end)
            }
        }
    }

    fn click(&mut self, at: (u16, u16)) -> Result<(), String> {
        let button = MouseButton::Left;
        self.send(&mouse(MouseEventKind::Down(button), at, KeyModifiers::NONE))?;
        self.send(&mouse(MouseEventKind::Up(button), at, KeyModifiers::NONE))
    }

    fn drag(&mut self, from: (u16, u16), to: (u16, u16)) -> Result<(), String> {
        let button = MouseButton::Left;
        self.send(&mouse(
            MouseEventKind::Down(button),
            from,
            KeyModifiers::NONE,
        ))?;
        self.send(&mouse(MouseEventKind::Drag(button), to, KeyModifiers::NONE))?;
        self.send(&mouse(MouseEventKind::Up(button), to, KeyModifiers::NONE))
    }

    fn send(&mut self, event: &Event) -> Result<(), String> {
        if !self.app.state().running {
            return Err("temoc-ui has already quit".into());
        }
        self.app
            .handle_event(event)
            .map_err(|_| NOT_IMPLEMENTED.to_owned())?;
        self.settle()
    }

    /// Waits for background work, then redraws the screen.
    fn settle(&mut self) -> Result<(), String> {
        let deadline = Instant::now() + RUN_TIMEOUT;
        self.app.update();
        while self.app.is_busy() {
            if Instant::now() > deadline {
                return Err("timed out waiting for temoc-ui".into());
            }
            sleep(POLL_INTERVAL);
            self.app.update();
        }
        self.output.append(&mut self.app.take_output());
        let mut drawn = false;
        self.terminal
            .draw(|frame| drawn = self.app.draw(frame).is_ok())
            .map_err(|e| e.to_string())?;
        self.drawn = drawn;
        Ok(())
    }

    fn locate(&self, text: &str) -> Result<Found, String> {
        screen::find(self.buffer(), self.buffer().area, text)
            .ok_or_else(|| format!("`{text}` is not on screen"))
    }

    fn buffer(&self) -> &Buffer {
        self.terminal.backend().buffer()
    }

    /// A snapshot of the application.
    pub fn state(&self) -> AppState {
        self.app.state()
    }

    /// Whether temoc-ui drew the screen.
    pub fn drawn(&self) -> bool {
        self.drawn
    }

    /// Whether `text` is on screen.
    pub fn shows(&self, text: &str) -> bool {
        screen::find(self.buffer(), self.buffer().area, text).is_some()
    }

    /// Whether `text` is inside `area` of the screen.
    pub fn shows_in(&self, area: Rect, text: &str) -> bool {
        screen::find(self.buffer(), area, text).is_some()
    }

    /// The payload of the last clipboard sequence written.
    pub fn clipboard_payload(&self) -> Option<&[u8]> {
        clipboard::last_payload(&self.output)
    }
}

fn mouse(kind: MouseEventKind, at: (u16, u16), modifiers: KeyModifiers) -> Event {
    Event::Mouse(MouseEvent {
        kind,
        column: at.0,
        row: at.1,
        modifiers,
    })
}
