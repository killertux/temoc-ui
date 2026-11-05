//! The fixtures used by the pages in `spec/`.

use crate::clipboard;
use crate::gui::{Played as GuiPlayed, Scene};
use crate::headless::{Played, Scenario};
use crate::screen;
use crate::session::{FakeTerminal, Outcome, Session};
use ratatui::style::Color;
use rust_slim::fixture;
use std::path::Path;
use temoc_ui::app::AppState;
use temoc_ui::graph::GraphState;
use temoc_ui::gui::egui::Theme;
use temoc_ui::gui::tags;
use temoc_ui::render::{self, Preview};
use temoc_ui::temoc::{self, Event, RunReport, TestFile, Transcript};
use temoc_ui::theme::Rgb;

pub const NONE: &str = "none";

fn yes_no(value: bool) -> String {
    if value { "yes" } else { "no" }.into()
}

fn or_none(value: Option<String>) -> String {
    value.unwrap_or_else(|| NONE.into())
}

/// A colour written like `#16181D`. Anything else is no colour.
fn hex_color(text: &str) -> Option<Rgb> {
    let digits = text.strip_prefix('#')?;
    let channel = |range| u8::from_str_radix(digits.get(range)?, 16).ok();
    Some(Rgb(channel(0..2)?, channel(2..4)?, channel(4..6)?))
}

fn list(items: &[String], separator: &str) -> String {
    if items.is_empty() {
        NONE.into()
    } else {
        items.join(separator)
    }
}

/// Runs a scenario with no interface. See `spec/writing-specs.md`.
#[derive(Default)]
pub struct Core {
    workspace: String,
    path: String,
    config: String,
    steps: String,
    played: Option<Played>,
}

#[fixture]
impl Core {
    #[slim(constructor)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn set_workspace(&mut self, workspace: String) {
        self.workspace = workspace;
    }

    pub fn set_path(&mut self, path: String) {
        self.path = path;
    }

    pub fn set_config(&mut self, config: String) {
        self.config = config;
    }

    pub fn set_steps(&mut self, steps: String) {
        self.steps = steps;
    }

    pub fn execute(&mut self) {
        self.played = Some(Played::of(
            &self.workspace,
            &self.path,
            &self.config,
            &self.steps,
        ));
    }

    #[must_use]
    pub fn document(&self) -> String {
        if matches!(self.played, Some(Played::Refused(_))) {
            return NONE.into();
        }
        self.read(|scenario| or_none(scenario.document()))
    }

    #[must_use]
    pub fn anchor(&self) -> String {
        self.read(|scenario| or_none(scenario.anchor()))
    }

    #[must_use]
    pub fn problem(&self) -> String {
        if let Some(Played::Refused(problem)) = &self.played {
            return problem.clone();
        }
        self.read(|scenario| or_none(scenario.problem()))
    }

    #[must_use]
    pub fn results(&self) -> String {
        self.read(|scenario| list(&scenario.results(), ", "))
    }

    #[must_use]
    pub fn first_result(&self) -> String {
        self.read(|scenario| or_none(scenario.results().first().cloned()))
    }

    #[must_use]
    pub fn tree(&self) -> String {
        self.read(|scenario| list(&scenario.tree(), ", "))
    }

    #[must_use]
    pub fn links(&self) -> String {
        self.read(|scenario| {
            let links = scenario.graph().link_names();
            let links: Vec<String> = links
                .iter()
                .map(|(from, to)| format!("{from} -> {to}"))
                .collect();
            list(&links, ", ")
        })
    }

    #[must_use]
    pub fn distance(&self) -> String {
        self.read(|scenario| {
            scenario.at_document(|graph, file| {
                graph
                    .distance(file)
                    .map_or_else(|| "not reachable".into(), |distance| distance.to_string())
            })
        })
    }

    #[must_use]
    pub fn links_out(&self) -> String {
        self.read(|scenario| scenario.at_document(|graph, file| list(&graph.links_out(file), ", ")))
    }

    #[must_use]
    pub fn links_in(&self) -> String {
        self.read(|scenario| scenario.at_document(|graph, file| list(&graph.links_in(file), ", ")))
    }

    #[must_use]
    pub fn file_result(&self) -> String {
        self.read(|scenario| or_none(scenario.outcome().file.map(String::from)))
    }

    #[must_use]
    pub fn table_results(&self) -> String {
        self.read(|scenario| {
            let tables = scenario.outcome().tables;
            let tables: Vec<String> = tables
                .iter()
                .map(|(table, result)| format!("{table}: {result}"))
                .collect();
            list(&tables, ", ")
        })
    }

    #[must_use]
    pub fn row_results(&self) -> String {
        self.read(|scenario| {
            let rows = scenario.outcome().rows;
            let rows: Vec<String> = rows
                .iter()
                .map(|(line, result)| format!("{line}: {result}"))
                .collect();
            list(&rows, ", ")
        })
    }

    #[must_use]
    pub fn failures(&self) -> String {
        self.read(|scenario| list(&scenario.outcome().failures, "; "))
    }

    #[must_use]
    pub fn temoc_arguments(&self) -> String {
        self.read(|scenario| {
            or_none(
                scenario
                    .session()
                    .temoc_arguments()
                    .map(|arguments| arguments.join(" ")),
            )
        })
    }

    #[must_use]
    pub fn running(&self) -> String {
        self.read(|scenario| or_none(scenario.session().running_file().map(String::from)))
    }
}

impl Core {
    fn read(&self, read: impl FnOnce(&Scenario) -> String) -> String {
        match &self.played {
            None => "scenario did not run".into(),
            Some(Played::Refused(_)) => "temoc-ui did not start".into(),
            Some(Played::Broken(problem)) => problem.clone(),
            Some(Played::Finished(scenario)) => read(scenario),
        }
    }
}

/// Runs a scenario against the GUI. See `spec/gui/writing-specs.md`.
#[derive(Default)]
pub struct Gui {
    workspace: String,
    arguments: String,
    actions: String,
    row: String,
    system_mode: String,
    played: Option<GuiPlayed>,
}

#[fixture]
impl Gui {
    #[slim(constructor)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn set_workspace(&mut self, workspace: String) {
        self.workspace = workspace;
    }

    pub fn set_arguments(&mut self, arguments: String) {
        self.arguments = arguments;
    }

    pub fn set_actions(&mut self, actions: String) {
        self.actions = actions;
    }

    pub fn set_row(&mut self, row: String) {
        self.row = row;
    }

    pub fn set_system_mode(&mut self, system_mode: String) {
        self.system_mode = system_mode;
    }

    pub fn execute(&mut self) {
        self.played = Some(GuiPlayed::of(
            &self.workspace,
            &self.arguments,
            &self.actions,
            &self.system_mode,
        ));
    }

    #[must_use]
    pub fn still_open(&self) -> String {
        match &self.played {
            Some(GuiPlayed::Refused(_)) => yes_no(false),
            _ => self.read(|scene| yes_no(scene.window().is_open())),
        }
    }

    #[must_use]
    pub fn status(&self) -> String {
        if let Some(GuiPlayed::Refused(message)) = &self.played {
            return message.clone();
        }
        self.read(|scene| or_none(scene.first_tagged(tags::STATUS)))
    }

    #[must_use]
    pub fn document(&self) -> String {
        self.read(|scene| or_none(scene.first_tagged(tags::DOCUMENT)))
    }

    #[must_use]
    pub fn banner(&self) -> String {
        self.read(|scene| or_none(scene.first_tagged(tags::BANNER)))
    }

    #[must_use]
    pub fn file_result(&self) -> String {
        self.read(|scene| or_none(scene.first_tagged(tags::FILE_RESULT)))
    }

    #[must_use]
    pub fn temoc_arguments(&self) -> String {
        self.read(|scene| {
            let arguments = scene.window().temoc_arguments();
            or_none(arguments.map(|arguments| arguments.join(" ")))
        })
    }

    #[must_use]
    pub fn search_open(&self) -> String {
        self.read(|scene| yes_no(scene.is_tagged(tags::SEARCH)))
    }

    #[must_use]
    pub fn results(&self) -> String {
        self.read(|scene| list(&scene.tagged(tags::SEARCH_RESULT), ", "))
    }

    #[must_use]
    pub fn tree_open(&self) -> String {
        self.read(|scene| yes_no(scene.is_tagged(tags::TREE)))
    }

    #[must_use]
    pub fn tree(&self) -> String {
        self.read(|scene| list(&scene.tree(), ", "))
    }

    #[must_use]
    pub fn tree_selection(&self) -> String {
        self.read(|scene| or_none(scene.tree_selection()))
    }

    #[must_use]
    pub fn tree_test_files(&self) -> String {
        self.read(|scene| list(&scene.tree_test_files(), ", "))
    }

    #[must_use]
    pub fn tree_marks(&self) -> String {
        self.read(|scene| list(&scene.tree_marks(), ", "))
    }

    #[must_use]
    pub fn tests_panel_open(&self) -> String {
        self.read(|scene| yes_no(scene.is_tagged(tags::TESTS_PANEL)))
    }

    #[must_use]
    pub fn tests_panel(&self) -> String {
        self.read(|scene| list(&scene.panel_tables(), ", "))
    }

    #[must_use]
    pub fn not_passed(&self) -> String {
        self.read(|scene| list(&scene.tagged(tags::NOT_PASSED), "; "))
    }

    #[must_use]
    pub fn last_command(&self) -> String {
        self.read(|scene| or_none(scene.first_tagged(tags::LAST_COMMAND)))
    }

    #[must_use]
    pub fn run_file(&self) -> String {
        self.read(|scene| or_none(scene.run_file().map(String::from)))
    }

    #[must_use]
    pub fn focused_table(&self) -> String {
        self.read(|scene| or_none(scene.focused_table()))
    }

    #[must_use]
    pub fn focused_row(&self) -> String {
        self.read(|scene| or_none(scene.focused_row()))
    }

    #[must_use]
    pub fn run_row(&self) -> String {
        self.read(|scene| list(&scene.run_rows(), ", "))
    }

    #[must_use]
    pub fn row_results(&self) -> String {
        self.read(|scene| list(&scene.row_results(), ", "))
    }

    #[must_use]
    pub fn marks(&self) -> String {
        self.read(|scene| list(&scene.tagged(tags::MARK), ", "))
    }

    #[must_use]
    pub fn note(&self) -> String {
        self.read(|scene| list(&scene.notes(&self.row), "; "))
    }

    #[must_use]
    pub fn table_verdicts(&self) -> String {
        self.read(|scene| list(&scene.tagged(tags::VERDICT), ", "))
    }

    #[must_use]
    pub fn help_open(&self) -> String {
        self.read(|scene| yes_no(scene.is_tagged(tags::HELP)))
    }

    #[must_use]
    pub fn mode(&self) -> String {
        self.read(|scene| match scene.mode() {
            Theme::Dark => "dark".into(),
            Theme::Light => "light".into(),
        })
    }
}

impl Gui {
    fn read(&self, read: impl FnOnce(&Scene) -> String) -> String {
        match &self.played {
            None => "scenario did not run".into(),
            Some(GuiPlayed::Refused(_)) => NONE.into(),
            Some(GuiPlayed::Broken(problem)) => problem.clone(),
            Some(GuiPlayed::Finished(scene)) => read(scene),
        }
    }
}

/// Runs a scenario against temoc-ui. See `spec/tui/writing-specs.md`.
#[derive(Default)]
pub struct Tui {
    workspace: String,
    arguments: String,
    keys: String,
    look_for: String,
    color_term: String,
    terminal_background: String,
    outcome: Option<Outcome>,
}

#[fixture]
impl Tui {
    #[slim(constructor)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn set_workspace(&mut self, workspace: String) {
        self.workspace = workspace;
    }

    pub fn set_arguments(&mut self, arguments: String) {
        self.arguments = arguments;
    }

    pub fn set_keys(&mut self, keys: String) {
        self.keys = keys;
    }

    pub fn set_look_for(&mut self, look_for: String) {
        self.look_for = look_for;
    }

    pub fn set_color_term(&mut self, color_term: String) {
        self.color_term = color_term;
    }

    pub fn set_terminal_background(&mut self, terminal_background: String) {
        self.terminal_background = terminal_background;
    }

    pub fn execute(&mut self) {
        let terminal = FakeTerminal {
            color_term: Some(self.color_term.clone()).filter(|term| !term.is_empty()),
            background: hex_color(&self.terminal_background),
        };
        self.outcome = Some(Outcome::of(
            &self.workspace,
            &self.arguments,
            &self.keys,
            terminal,
        ));
    }

    #[must_use]
    pub fn theme(&self) -> String {
        self.read_state(|state| state.theme.name.to_owned())
    }

    #[must_use]
    pub fn foreground(&self) -> String {
        self.read_colors(|(foreground, _)| screen::color_name(foreground))
    }

    #[must_use]
    pub fn background(&self) -> String {
        self.read_colors(|(_, background)| screen::color_name(background))
    }

    #[must_use]
    pub fn contrast(&self) -> String {
        self.read_colors(|(foreground, background)| {
            or_none(screen::contrast(foreground, background))
        })
    }

    #[must_use]
    pub fn bar_against_page(&self) -> String {
        self.read_session(|session| or_none(session.bar_against_page()))
    }

    #[must_use]
    pub fn unpainted_cells(&self) -> String {
        self.read_session(|session| session.unpainted_cells().to_string())
    }

    #[must_use]
    pub fn document(&self) -> String {
        self.read_state(|state| or_none(state.document.clone()))
    }

    #[must_use]
    pub fn visible(&self) -> String {
        self.read_session(|session| yes_no(session.shows(&self.look_for)))
    }

    #[must_use]
    pub fn highlight(&self) -> String {
        self.read_session(|session| {
            session
                .highlight(&self.look_for)
                .map_or_else(|problem| problem, String::from)
        })
    }

    #[must_use]
    pub fn next_line(&self) -> String {
        self.read_session(|session| {
            session.next_line(&self.look_for).map_or_else(
                |problem| problem,
                |line| or_none(Some(line).filter(|line| !line.is_empty())),
            )
        })
    }

    #[must_use]
    pub fn status(&self) -> String {
        match &self.outcome {
            Some(Outcome::Refused(message)) => message.clone(),
            _ => self.read_state(|state| or_none(state.status.clone())),
        }
    }

    #[must_use]
    pub fn still_open(&self) -> String {
        self.read_state(|state| yes_no(state.running))
    }

    #[must_use]
    pub fn focused_link(&self) -> String {
        self.read_state(|state| or_none(state.focused_link.clone()))
    }

    #[must_use]
    pub fn search_open(&self) -> String {
        self.read_state(|state| yes_no(state.search_open))
    }

    #[must_use]
    pub fn results(&self) -> String {
        self.read_state(|state| list(&state.search_results, ", "))
    }

    #[must_use]
    pub fn first_result(&self) -> String {
        self.read_state(|state| or_none(state.search_results.first().cloned()))
    }

    #[must_use]
    pub fn focused_table(&self) -> String {
        self.read_state(|state| or_none(state.focused_table.clone()))
    }

    #[must_use]
    pub fn focused_row(&self) -> String {
        self.read_state(|state| or_none(state.focused_row.map(|line| line.to_string())))
    }

    #[must_use]
    pub fn row_results(&self) -> String {
        self.read_state(|state| {
            let results: Vec<String> = state
                .row_results
                .iter()
                .map(|(line, result)| format!("{line}: {result}"))
                .collect();
            list(&results, ", ")
        })
    }

    #[must_use]
    pub fn table_results(&self) -> String {
        self.read_state(|state| {
            let results: Vec<String> = state
                .table_results
                .iter()
                .map(|(table, result)| format!("{table}: {result}"))
                .collect();
            list(&results, ", ")
        })
    }

    #[must_use]
    pub fn file_result(&self) -> String {
        self.read_state(|state| or_none(state.file_result.clone()))
    }

    #[must_use]
    pub fn failures(&self) -> String {
        self.read_state(|state| list(&state.failures, "; "))
    }

    #[must_use]
    pub fn temoc_arguments(&self) -> String {
        self.read_state(|state| {
            or_none(
                state
                    .temoc_arguments
                    .as_ref()
                    .map(|arguments| arguments.join(" ")),
            )
        })
    }

    #[must_use]
    pub fn tree_open(&self) -> String {
        self.read_state(|state| yes_no(state.tree.is_some()))
    }

    #[must_use]
    pub fn tree(&self) -> String {
        self.read_state(|state| list(state.tree.as_deref().unwrap_or_default(), ", "))
    }

    #[must_use]
    pub fn tree_selection(&self) -> String {
        self.read_state(|state| or_none(state.tree_selection.clone()))
    }

    #[must_use]
    pub fn graph_open(&self) -> String {
        self.read_state(|state| yes_no(state.graph.is_some()))
    }

    #[must_use]
    pub fn columns(&self) -> String {
        self.read_graph(|graph| {
            let columns: Vec<String> = graph.columns.iter().map(|c| c.join(", ")).collect();
            list(&columns, "; ")
        })
    }

    #[must_use]
    pub fn links(&self) -> String {
        self.read_graph(|graph| {
            let links: Vec<String> = graph
                .links
                .iter()
                .map(|(from, to)| format!("{from} -> {to}"))
                .collect();
            list(&links, ", ")
        })
    }

    #[must_use]
    pub fn selected_file(&self) -> String {
        self.read_graph(|graph| or_none(graph.selected.clone()))
    }

    #[must_use]
    pub fn distance(&self) -> String {
        self.read_graph(|graph| {
            graph
                .distance
                .map_or_else(|| "not reachable".into(), |distance| distance.to_string())
        })
    }

    #[must_use]
    pub fn links_out(&self) -> String {
        self.read_graph(|graph| list(&graph.links_out, ", "))
    }

    #[must_use]
    pub fn links_in(&self) -> String {
        self.read_graph(|graph| list(&graph.links_in, ", "))
    }

    #[must_use]
    pub fn in_graph(&self) -> String {
        self.read_session(|session| {
            session.state().graph.map_or_else(
                || yes_no(false),
                |graph| yes_no(session.shows_in(graph.area, &self.look_for)),
            )
        })
    }

    #[must_use]
    pub fn selection(&self) -> String {
        self.read_state(|state| or_none(state.selection.as_deref().map(show_line_breaks)))
    }

    #[must_use]
    pub fn clipboard(&self) -> String {
        self.read_session(|session| match session.clipboard_payload() {
            None => NONE.into(),
            Some(payload) => clipboard::decode(payload)
                .map_or_else(|problem| problem, |text| show_line_breaks(&text)),
        })
    }

    #[must_use]
    pub fn clipboard_sequence(&self) -> String {
        self.read_session(|session| or_none(session.clipboard_payload().map(clipboard::describe)))
    }
}

impl Tui {
    fn read_session(&self, read: impl FnOnce(&Session) -> String) -> String {
        match &self.outcome {
            None => "scenario did not run".into(),
            Some(Outcome::Refused(_)) => "temoc-ui did not open".into(),
            Some(Outcome::Broken(problem)) => problem.clone(),
            Some(Outcome::Finished(session)) => read(session),
        }
    }

    fn read_colors(&self, read: impl FnOnce((Color, Color)) -> String) -> String {
        self.read_session(|session| {
            session
                .colors(&self.look_for)
                .map_or_else(|problem| problem, read)
        })
    }

    /// Reads the application's state. temoc-ui refusing to open is a valid
    /// result, and looks like an application that closed with nothing open.
    fn read_state(&self, read: impl FnOnce(&AppState) -> String) -> String {
        match &self.outcome {
            Some(Outcome::Refused(_)) => read(&AppState::default()),
            _ => self.read_session(|session| read(&session.state())),
        }
    }

    fn read_graph(&self, read: impl FnOnce(&GraphState) -> String) -> String {
        self.read_state(|state| state.graph.as_ref().map_or_else(|| NONE.into(), read))
    }
}

fn show_line_breaks(text: &str) -> String {
    text.replace('\n', "\\n")
}

/// Renders a piece of markdown by itself.
#[derive(Default)]
pub struct MarkdownRendering {
    markdown: String,
}

#[fixture]
impl MarkdownRendering {
    #[slim(constructor)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_markdown(&mut self, markdown: String) {
        self.markdown = markdown;
    }

    #[must_use]
    pub fn text(&self) -> String {
        self.read("nothing", |preview| preview.text)
    }

    #[must_use]
    pub fn style(&self) -> String {
        self.read(NONE, |preview| preview.style)
    }
}

impl MarkdownRendering {
    fn read(&self, nothing: &str, read: impl FnOnce(Preview) -> String) -> String {
        match render::preview(&self.markdown) {
            Err(problem) => problem,
            Ok(None) => nothing.into(),
            Ok(Some(preview)) => read(preview),
        }
    }
}

/// Looks for temoc tests in a file from the `detection` sample.
#[derive(Default)]
pub struct TemocDetection {
    file: String,
}

#[fixture]
impl TemocDetection {
    #[slim(constructor)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_file(&mut self, file: String) {
        self.file = file;
    }

    #[must_use]
    pub fn test_file(&self) -> String {
        self.read(|found| yes_no(found.is_test_file))
    }

    #[must_use]
    pub fn tables(&self) -> String {
        self.read(|found| found.tables.len().to_string())
    }
}

impl TemocDetection {
    fn read(&self, read: impl FnOnce(TestFile) -> String) -> String {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../samples/detection")
            .join(&self.file);
        temoc::inspect_file(&path).map_or_else(|error| error.to_string(), read)
    }
}

/// Reads temoc's output by itself.
#[derive(Default)]
pub struct TemocOutput {
    events: String,
    stderr: String,
    exit_code: i32,
}

#[fixture]
impl TemocOutput {
    #[slim(constructor)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn set_events(&mut self, events: String) {
        self.events = events;
    }

    pub fn set_stderr(&mut self, stderr: String) {
        self.stderr = stderr;
    }

    pub fn set_exit_code(&mut self, exit_code: i32) {
        self.exit_code = exit_code;
    }

    #[must_use]
    pub fn result(&self) -> String {
        self.read().1.result.to_string()
    }

    #[must_use]
    pub fn message(&self) -> String {
        or_none(self.read().1.message)
    }

    #[must_use]
    pub fn failures(&self) -> String {
        let failures: Vec<String> = self
            .read()
            .0
            .iter()
            .flat_map(Event::failures)
            .map(ToString::to_string)
            .collect();
        list(&failures, "; ")
    }
}

impl TemocOutput {
    fn read(&self) -> (Vec<Event>, RunReport) {
        let mut transcript = Transcript::default();
        let events = self.events.replace("\\n", "\n");
        let events = events.lines().filter_map(|line| transcript.read(line));
        let events = events.collect();
        (events, transcript.report(&self.stderr, self.exit_code))
    }
}
