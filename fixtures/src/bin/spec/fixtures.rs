//! The fixtures used by the pages in `spec/`.

use crate::clipboard;
use crate::session::{Outcome, Session};
use rust_slim::fixture;
use std::path::Path;
use temoc_ui::app::AppState;
use temoc_ui::graph::GraphState;
use temoc_ui::render::{self, Preview};
use temoc_ui::temoc::{self, RunReport, TestFile};

const NONE: &str = "none";

fn yes_no(value: bool) -> String {
    if value { "yes" } else { "no" }.into()
}

fn or_none(value: Option<String>) -> String {
    value.unwrap_or_else(|| NONE.into())
}

fn list(items: &[String], separator: &str) -> String {
    if items.is_empty() {
        NONE.into()
    } else {
        items.join(separator)
    }
}

/// Runs a scenario against temoc-ui. See `spec/writing-specs.md`.
#[derive(Default)]
pub struct Tui {
    workspace: String,
    arguments: String,
    keys: String,
    look_for: String,
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

    pub fn execute(&mut self) {
        self.outcome = Some(Outcome::of(&self.workspace, &self.arguments, &self.keys));
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
    stdout: String,
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

    pub fn set_stdout(&mut self, stdout: String) {
        self.stdout = stdout;
    }

    pub fn set_stderr(&mut self, stderr: String) {
        self.stderr = stderr;
    }

    pub fn set_exit_code(&mut self, exit_code: i32) {
        self.exit_code = exit_code;
    }

    #[must_use]
    pub fn result(&self) -> String {
        self.read().result.to_string()
    }

    #[must_use]
    pub fn failures(&self) -> String {
        let failures: Vec<String> = self
            .read()
            .failures
            .iter()
            .map(ToString::to_string)
            .collect();
        list(&failures, "; ")
    }
}

impl TemocOutput {
    fn read(&self) -> RunReport {
        let stdout = self.stdout.replace("\\n", "\n");
        temoc::read_output(&stdout, &self.stderr, self.exit_code)
    }
}
