//! The temoc-ui command.

use crossterm::event;
use ratatui::DefaultTerminal;
use std::env;
use std::io::{self, Write};
use std::process::ExitCode;
use std::time::Duration;
use temoc_ui::NotImplemented;
use temoc_ui::app::{App, Launch, StartError};

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
    let mut app = App::start(&launch).map_err(|error| match error {
        StartError::Refused(message) => message,
        StartError::NotImplemented => "That option is not built yet".to_owned(),
    })?;
    let mut terminal = ratatui::try_init().map_err(|error| error.to_string())?;
    let result = show(&mut app, &mut terminal);
    let restored = ratatui::try_restore();
    result.and(restored).map_err(|error| error.to_string())
}

fn show(app: &mut App, terminal: &mut DefaultTerminal) -> io::Result<()> {
    while app.state().running {
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
