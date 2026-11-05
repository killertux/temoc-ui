//! The temoc-ui command.

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use ratatui::DefaultTerminal;
use std::env;
use std::io::{self, Write};
use std::process::ExitCode;
use std::time::Duration;
use temoc_ui::NotImplemented;
use temoc_ui::app::{self, App, Launch};
use temoc_ui::terminal;

/// How often background work is picked up while no input arrives.
const TICK: Duration = Duration::from_millis(100);

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let gui = app::wants_gui(&arguments);
    // There is no need to bother the terminal when the theme is already chosen,
    // or when it is not what temoc-ui will draw on.
    let chosen = gui || arguments.iter().any(|argument| argument == "--theme");
    let launch = Launch {
        current_dir: env::current_dir().map_err(|error| error.to_string())?,
        arguments,
        extra_path: Vec::new(),
        color_term: env::var_os("COLORTERM").map(|term| term.to_string_lossy().into_owned()),
        background: if chosen { None } else { terminal::background() },
    };
    if gui {
        return open_gui(&launch);
    }
    let mut app = App::start(&launch)?;
    let mut terminal = ratatui::try_init().map_err(|error| error.to_string())?;
    let result =
        execute!(io::stdout(), EnableMouseCapture).and_then(|()| show(&mut app, &mut terminal));
    let released = execute!(io::stdout(), DisableMouseCapture);
    let restored = ratatui::try_restore();
    result
        .and(released)
        .and(restored)
        .map_err(|error| error.to_string())
}

#[cfg(feature = "gui")]
fn open_gui(launch: &Launch) -> Result<(), String> {
    temoc_ui::gui::open(launch)
}

#[cfg(not(feature = "gui"))]
fn open_gui(_launch: &Launch) -> Result<(), String> {
    Err("This build has no GUI. Rebuild it with the gui feature.".into())
}

fn show(app: &mut App, terminal: &mut DefaultTerminal) -> io::Result<()> {
    while app.is_running() {
        app.update();
        terminal.draw(|frame| app.draw(frame))?;
        let mut stdout = io::stdout();
        stdout.write_all(&app.take_output())?;
        stdout.flush()?;
        if event::poll(TICK)? {
            match app.handle_event(&event::read()?) {
                // Input that is not built yet does nothing.
                Ok(()) | Err(NotImplemented) => {}
            }
        }
    }
    Ok(())
}
