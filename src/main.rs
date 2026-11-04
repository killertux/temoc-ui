//! The temoc-ui command.

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use ratatui::DefaultTerminal;
use std::env;
use std::io::{self, Write};
use std::process::ExitCode;
use std::time::Duration;
use temoc_ui::NotImplemented;
use temoc_ui::app::{App, Launch};

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
    let launch = Launch {
        current_dir: env::current_dir().map_err(|error| error.to_string())?,
        arguments: env::args().skip(1).collect(),
        extra_path: Vec::new(),
    };
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
