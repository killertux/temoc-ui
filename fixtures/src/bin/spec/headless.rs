//! Scenarios for the `Core` fixture: temoc-ui with no interface at all.

use crate::disk::{Change, Sandbox, wait_for_change};
use crate::fixtures::NONE;
use crate::session::{NOT_IMPLEMENTED, example_binaries, samples};
use std::path::PathBuf;
use std::thread::sleep;
use std::time::{Duration, Instant};
use temoc_ui::history::History;
use temoc_ui::links::LinkGraph;
use temoc_ui::search::Search;
use temoc_ui::session::{self, Opened, Outcome, Session, Start};
use temoc_ui::tree::{FileTree, Row};

const RUN_TIMEOUT: Duration = Duration::from_mins(1);
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// One of the steps described in `spec/writing-specs.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Step {
    Open(String),
    Follow(String),
    Back,
    Forward,
    Search(String),
    OnlyTestFiles,
    OpenResult(usize),
    RunFile,
    RunTable(usize),
    RunRow(usize),
    WaitForRow(usize),
    Change(Change),
}

impl Step {
    fn read(text: &str) -> Result<Self, String> {
        let number = |text: &str| {
            let refuse = |_| format!("`{text}` is not a number");
            text.parse::<usize>().map_err(refuse)
        };
        let (word, rest) = text.split_once(' ').unwrap_or((text, ""));
        match (word, rest) {
            ("back", "") => Ok(Self::Back),
            ("forward", "") => Ok(Self::Forward),
            ("search", query) => Ok(Self::Search(query.into())),
            ("only", "test files") => Ok(Self::OnlyTestFiles),
            ("follow", target) if !target.is_empty() => Ok(Self::Follow(target.into())),
            ("run", "file") => Ok(Self::RunFile),
            ("run", rest) => match rest.split_once(' ') {
                Some(("table", line)) => Ok(Self::RunTable(number(line)?)),
                Some(("row", line)) => Ok(Self::RunRow(number(line)?)),
                _ => Err(format!("unknown step: {text}")),
            },
            ("open", rest) => match rest.strip_prefix("result ") {
                Some(result) => Ok(Self::OpenResult(number(result)?)),
                None if rest.is_empty() => Err(format!("unknown step: {text}")),
                None => Ok(Self::Open(rest.into())),
            },
            ("wait", rest) => match rest.strip_prefix("for row ") {
                Some(line) => Ok(Self::WaitForRow(number(line)?)),
                None => Err(format!("unknown step: {text}")),
            },
            ("write", rest) => match rest.split_once(" from ") {
                Some((file, version)) => Ok(Self::Change(Change::Write {
                    file: file.into(),
                    version: version.into(),
                })),
                None => Err(format!("unknown step: {text}")),
            },
            ("replace", rest) => match rest.split_once(" with ") {
                Some((file, version)) => Ok(Self::Change(Change::Replace {
                    file: file.into(),
                    version: version.into(),
                })),
                None => Err(format!("unknown step: {text}")),
            },
            ("rename", rest) => match rest.split_once(" to ") {
                Some((from, to)) => Ok(Self::Change(Change::Rename {
                    from: from.into(),
                    to: to.into(),
                })),
                None => Err(format!("unknown step: {text}")),
            },
            ("delete", file) if !file.is_empty() => Ok(Self::Change(Change::Delete(file.into()))),
            _ => Err(format!("unknown step: {text}")),
        }
    }

    fn read_all(text: &str) -> Result<Vec<Self>, String> {
        text.split(',')
            .map(str::trim)
            .filter(|step| !step.is_empty())
            .map(Self::read)
            .collect()
    }
}

/// An open document, and the heading it was opened at.
struct Visit {
    opened: Opened,
    anchor: Option<String>,
    /// Whether its file was deleted after it was read.
    deleted: bool,
}

/// A temoc-ui with no interface, and what has been done to it.
pub struct Scenario {
    session: Session,
    documents: History<Visit>,
    search: Option<Search>,
    problem: Option<String>,
    /// Whether steps wait for a test run to finish.
    patient: bool,
    /// The copy of the workspace that steps change, if they change it. It
    /// goes last, so that temoc-ui is gone before it is.
    _sandbox: Option<Sandbox>,
}

impl Scenario {
    fn start(session: Session, sandbox: Option<Sandbox>) -> Self {
        let mut scenario = Self {
            session,
            documents: History::default(),
            search: None,
            problem: None,
            patient: true,
            _sandbox: sandbox,
        };
        if let Some(file) = scenario.session.first_file().map(String::from) {
            scenario.problem = scenario.visit(&file, None).err();
        }
        scenario
    }

    fn play(&mut self, steps: &[Step]) -> Result<(), String> {
        let mut steps = steps.iter().peekable();
        while let Some(step) = steps.next() {
            if matches!(steps.peek(), Some(Step::WaitForRow(_))) {
                self.patient = false;
            }
            if let Step::WaitForRow(line) = step {
                self.wait_for_row(*line)?;
            } else {
                self.problem = self.take(step).err();
                self.settle()?;
            }
        }
        Ok(())
    }

    fn take(&mut self, step: &Step) -> Result<(), String> {
        match step {
            Step::Open(file) => self.visit(file, None),
            Step::Follow(target) => self.follow(target),
            Step::Back => {
                if let Some(visit) = self.documents.behind() {
                    self.session.still_there(visit.opened.file())?;
                }
                self.documents.back();
                Ok(())
            }
            Step::Forward => {
                if let Some(visit) = self.documents.ahead() {
                    self.session.still_there(visit.opened.file())?;
                }
                self.documents.forward();
                Ok(())
            }
            Step::Search(query) => {
                let mut search = Search::open(self.session.workspace(), false);
                query.chars().for_each(|c| search.type_character(c));
                self.search = Some(search);
                Ok(())
            }
            Step::OnlyTestFiles => {
                if let Some(search) = &mut self.search {
                    search.toggle_only_tests();
                }
                Ok(())
            }
            Step::OpenResult(number) => self.open_result(*number),
            Step::RunFile => {
                let opened = self.documents.current().map(|visit| &visit.opened);
                self.session.run_file(opened)
            }
            Step::RunTable(line) => {
                let visit = self.documents.current();
                self.session
                    .run_table(visit.map(|visit| (&visit.opened, *line)))
            }
            Step::RunRow(line) => {
                let visit = self.documents.current();
                self.session
                    .run_row(visit.map(|visit| (&visit.opened, *line)))
            }
            Step::WaitForRow(_) => Err(NOT_IMPLEMENTED.to_owned()),
            Step::Change(change) => self.change(change),
        }
    }

    /// Changes a file, and waits for temoc-ui to notice. Says if the open
    /// document is gone.
    fn change(&mut self, change: &Change) -> Result<(), String> {
        let before = self.session.changes_seen();
        change.apply(self.session.workspace())?;
        wait_for_change(before, || {
            self.update();
            Ok(self.session.changes_seen())
        })?;
        match self.documents.current() {
            Some(visit) if visit.deleted => Err(session::file_deleted(visit.opened.file())),
            _ => Ok(()),
        }
    }

    /// Shows the files as they are now.
    fn follow_changes(&mut self) {
        let changes = self.session.changes();
        if changes.is_empty() {
            return;
        }
        for visit in self.documents.visits_mut() {
            if changes.covers(visit.opened.file()) {
                match self.session.reread(visit.opened.file()) {
                    Ok(Some(opened)) => {
                        visit.opened = opened;
                        visit.deleted = false;
                    }
                    Ok(None) => visit.deleted = true,
                    Err(problem) => self.problem = Some(problem),
                }
            }
        }
        if let Some(search) = &mut self.search {
            search.reload(self.session.workspace());
        }
    }

    fn visit(&mut self, file: &str, anchor: Option<String>) -> Result<(), String> {
        let opened = self.session.read(file)?;
        self.documents.visit(Visit {
            opened,
            anchor,
            deleted: false,
        });
        Ok(())
    }

    fn follow(&mut self, target: &str) -> Result<(), String> {
        let document = self.document().unwrap_or_default();
        let place = self.session.follow(&document, target)?;
        self.visit(&place.file, place.anchor)
    }

    fn open_result(&mut self, number: usize) -> Result<(), String> {
        let results = self.search.as_ref().map_or(&[][..], Search::results);
        let Some(hit) = results.get(number.saturating_sub(1)).cloned() else {
            return Ok(());
        };
        self.visit(&hit.file, hit.heading.map(|heading| heading.anchor))
    }

    /// Picks up files that changed, the progress of the run, and what stopped it.
    fn update(&mut self) {
        self.follow_changes();
        if let Some(problem) = self.session.update() {
            self.problem = Some(problem);
        }
    }

    /// Waits for the run, if steps do.
    fn settle(&mut self) -> Result<(), String> {
        let deadline = Instant::now() + RUN_TIMEOUT;
        self.update();
        while self.patient && self.session.is_busy() {
            if Instant::now() > deadline {
                return Err("timed out waiting for temoc".into());
            }
            sleep(POLL_INTERVAL);
            self.update();
        }
        Ok(())
    }

    fn wait_for_row(&mut self, line: usize) -> Result<(), String> {
        let deadline = Instant::now() + RUN_TIMEOUT;
        loop {
            self.update();
            let rows = self.outcome().rows;
            if rows.iter().any(|row| *row == (line, "running".to_owned())) {
                return Ok(());
            }
            if Instant::now() > deadline {
                return Err(format!("timed out waiting for row {line}"));
            }
            sleep(POLL_INTERVAL);
        }
    }

    pub fn document(&self) -> Option<String> {
        let visit = self.documents.current()?;
        Some(visit.opened.file().to_owned())
    }

    pub fn outcome(&self) -> Outcome {
        let visit = self.documents.current();
        visit.map_or_else(Outcome::default, |visit| {
            self.session.outcome(&visit.opened)
        })
    }

    pub fn results(&self) -> Vec<String> {
        let results = self.search.as_ref().map_or(&[][..], Search::results);
        results.iter().map(ToString::to_string).collect()
    }

    /// The headings of the open document, in order.
    pub fn headings(&self) -> Vec<String> {
        let visit = self.documents.current();
        let headings = visit.map(|visit| visit.opened.document().headings());
        let headings = headings.unwrap_or_default().into_iter();
        headings.map(|heading| heading.text).collect()
    }

    /// The heading the open document was opened at.
    pub fn anchor(&self) -> Option<String> {
        self.documents.current()?.anchor.clone()
    }

    /// Why the last step was refused, or what stopped the last run.
    pub fn problem(&self) -> Option<String> {
        self.problem.clone()
    }

    /// The session behind the scenario.
    pub fn session(&self) -> &Session {
        &self.session
    }

    /// The rows of the file tree.
    pub fn tree(&self) -> Vec<String> {
        let document = self.document();
        let tree = FileTree::open(self.session.workspace(), document.as_deref());
        tree.rows().iter().map(tree_row).collect()
    }

    pub fn graph(&self) -> LinkGraph {
        LinkGraph::read(self.session.workspace())
    }

    /// Reads the graph at the open document.
    pub fn at_document(&self, read: impl FnOnce(&LinkGraph, usize) -> String) -> String {
        let graph = self.graph();
        let file = self.document().and_then(|file| graph.position(&file));
        file.map_or_else(|| NONE.into(), |file| read(&graph, file))
    }
}

fn tree_row(row: &Row) -> String {
    if row.is_folder() {
        format!("{}/", row.name)
    } else {
        row.name.clone()
    }
}

/// What happened when a scenario ran.
pub enum Played {
    /// temoc-ui refused to start, with its message.
    Refused(String),
    /// The scenario itself could not be carried out.
    Broken(String),
    /// The scenario ran, and this is the result.
    Finished(Box<Scenario>),
}

impl Played {
    /// Starts temoc-ui in a sample workspace and takes the steps.
    pub fn of(workspace: &str, path: &str, config: &str, steps: &str) -> Self {
        let given = |text: &str| Some(text.to_owned()).filter(|text| !text.is_empty());
        let steps = match Step::read_all(steps) {
            Ok(steps) => steps,
            Err(problem) => return Self::Broken(problem),
        };
        // Steps that change files work on a copy of the workspace.
        let changes = steps.iter().any(|step| matches!(step, Step::Change(_)));
        let sandbox = match changes.then(|| Sandbox::of(workspace)).transpose() {
            Ok(sandbox) => sandbox,
            Err(problem) => return Self::Broken(problem),
        };
        let folder = sandbox.as_ref().map(Sandbox::folder);
        let start = Start {
            current_dir: folder.map_or_else(|| samples().join(workspace), PathBuf::from),
            path: given(path),
            config: given(config),
            extra_path: example_binaries().into_iter().collect(),
        };
        let session = match Session::start(&start) {
            Ok(session) => session,
            Err(problem) => return Self::Refused(problem),
        };
        let mut scenario = Scenario::start(session, sandbox);
        let played = scenario.play(&steps);
        match played {
            Ok(()) => Self::Finished(Box::new(scenario)),
            Err(problem) => Self::Broken(problem),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_are_read_in_order_and_empty_means_none() {
        assert_eq!(Step::read_all(""), Ok(Vec::new()));
        assert_eq!(
            Step::read_all("run file, wait for row 13, back"),
            Ok(vec![Step::RunFile, Step::WaitForRow(13), Step::Back])
        );
    }

    #[test]
    fn a_search_may_have_no_text() {
        assert_eq!(Step::read("search"), Ok(Step::Search(String::new())));
        assert_eq!(Step::read("search a b"), Ok(Step::Search("a b".into())));
    }

    #[test]
    fn open_means_a_file_unless_it_says_result() {
        assert_eq!(Step::read("open a.md"), Ok(Step::Open("a.md".into())));
        assert_eq!(Step::read("open result 2"), Ok(Step::OpenResult(2)));
    }

    #[test]
    fn a_step_that_makes_no_sense_is_refused() {
        for step in [
            "jump",
            "run",
            "run tables 1",
            "open",
            "follow",
            "wait 3",
            "back up",
            "write a.md",
            "replace a.md by b.md",
            "rename a.md",
            "delete",
        ] {
            assert_eq!(Step::read(step), Err(format!("unknown step: {step}")));
        }
    }

    #[test]
    fn steps_that_change_files() {
        assert_eq!(
            Step::read_all("write a/b.md from c.md, replace a.md with b.md"),
            Ok(vec![
                Step::Change(Change::Write {
                    file: "a/b.md".into(),
                    version: "c.md".into()
                }),
                Step::Change(Change::Replace {
                    file: "a.md".into(),
                    version: "b.md".into()
                }),
            ])
        );
        assert_eq!(
            Step::read_all("delete a.md, rename a.md to b.md"),
            Ok(vec![
                Step::Change(Change::Delete("a.md".into())),
                Step::Change(Change::Rename {
                    from: "a.md".into(),
                    to: "b.md".into()
                }),
            ])
        );
    }

    #[test]
    fn a_line_must_be_a_number() {
        let refused = Err("`x` is not a number".to_owned());
        assert_eq!(Step::read("run row x"), refused);
        assert_eq!(Step::read("run table x"), refused);
        assert_eq!(Step::read("open result x"), refused);
        assert_eq!(Step::read("wait for row x"), refused);
    }
}
