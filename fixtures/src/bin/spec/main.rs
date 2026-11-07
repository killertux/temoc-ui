//! The Slim server that runs the specification in `spec/`.
//!
//! See `spec/writing-specs.md` for how the fixtures are used.

mod clipboard;
mod disk;
mod fixtures;
mod gui;
mod headless;
mod keys;
mod screen;
mod session;

use fixtures::{Core, Gui, MarkdownRendering, TemocDetection, TemocOutput, Tui};
use rust_slim::PortSlimServer;
use std::env::args;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut server = PortSlimServer::listen_from_args(args().skip(1))?;
    server.add_fixture::<Core>();
    server.add_fixture::<Tui>();
    server.add_fixture::<Gui>();
    server.add_fixture::<MarkdownRendering>();
    server.add_fixture::<TemocDetection>();
    server.add_fixture::<TemocOutput>();
    server.run()?;
    Ok(())
}
