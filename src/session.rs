//! What temoc-ui does, whatever its interface is: the workspace, its files, and
//! the runs of temoc.

use crate::document::Document;
use crate::run::{FileResults, NOT_RUN, Progress, Request, Run};
use crate::temoc::{RunReport, TestFile, TestTable};
use crate::workspace::{self, Place};
use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};

/// The configuration file temoc-ui looks for in the workspace.
pub const CONFIGURATION: &str = "temoc.toml";

/// What a session starts from.
#[derive(Debug, Clone, Default)]
pub struct Start {
    /// The folder temoc-ui was started from.
    pub current_dir: PathBuf,
    /// The path to open.
    pub path: Option<String>,
    /// The configuration file to use, as written.
    pub config: Option<String>,
    /// Folders to search ahead of `PATH` when starting temoc.
    pub extra_path: Vec<PathBuf>,
}

/// A file of the workspace that has been read.
#[derive(Debug, Clone)]
pub struct Opened {
    file: String,
    document: Document,
    tests: TestFile,
}

impl Opened {
    /// A document that goes by this name in the workspace.
    #[must_use]
    pub fn new(file: String, document: Document) -> Self {
        let tests = document.tests();
        Self {
            file,
            document,
            tests,
        }
    }

    /// The file, relative to the workspace.
    #[must_use]
    pub fn file(&self) -> &str {
        &self.file
    }

    /// Its markdown.
    #[must_use]
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Whether temoc would treat this file as a test file.
    #[must_use]
    pub fn is_test_file(&self) -> bool {
        self.tests.is_test_file
    }

    /// The tables that can run, in document order.
    #[must_use]
    pub fn tables(&self) -> &[TestTable] {
        &self.tests.tables
    }
}

/// What is known about the tests of a file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Outcome {
    /// The result for the whole file, if it has tests.
    pub file: Option<&'static str>,
    /// Each table's name and result, in document order.
    pub tables: Vec<(String, String)>,
    /// The line and result of each row that has one, in document order.
    pub rows: Vec<(usize, String)>,
    /// Each failure, with its table and line.
    pub failures: Vec<String>,
}

/// One workspace, and what has happened in it so far.
#[derive(Debug)]
pub struct Session {
    workspace: PathBuf,
    /// What opens first, if anything does.
    first_file: Option<String>,
    /// The folder temoc-ui was started from.
    start_folder: PathBuf,
    /// The configuration file given, as written.
    config: Option<String>,
    /// Folders to search ahead of `PATH` when starting temoc.
    extra_path: Vec<PathBuf>,
    /// The temoc process at work.
    run: Option<Run>,
    /// What each file's runs have reported.
    results: HashMap<String, FileResults>,
    /// The arguments of the last temoc command.
    temoc_arguments: Option<Vec<String>>,
}

impl Session {
    /// Finds the workspace.
    ///
    /// # Errors
    ///
    /// Fails with a message for the user when the path cannot be opened.
    pub fn start(start: &Start) -> Result<Self, String> {
        let opening = workspace::locate(&start.current_dir, start.path.as_deref())?;
        Ok(Self {
            workspace: opening.workspace,
            first_file: opening.file,
            start_folder: start.current_dir.clone(),
            config: start.config.clone(),
            extra_path: start.extra_path.clone(),
            run: None,
            results: HashMap::new(),
            temoc_arguments: None,
        })
    }

    /// The workspace folder.
    #[must_use]
    pub fn workspace(&self) -> &Path {
        &self.workspace
    }

    /// What opens first, relative to the workspace. Without one, it is up to
    /// the interface to offer the files of the workspace.
    #[must_use]
    pub fn first_file(&self) -> Option<&str> {
        self.first_file.as_deref()
    }

    /// Reads a file of the workspace.
    ///
    /// # Errors
    ///
    /// Fails with a message for the user when it cannot be read.
    pub fn read(&self, file: &str) -> Result<Opened, String> {
        let refuse = |error: &dyn Display| format!("Cannot read {file}: {error}");
        let text = fs::read_to_string(self.workspace.join(file)).map_err(|e| refuse(&e))?;
        let document = Document::parse(&text).map_err(|e| refuse(&e))?;
        Ok(Opened::new(file.to_owned(), document))
    }

    /// Where a link in `document` leads.
    ///
    /// # Errors
    ///
    /// Fails with a message for the user when the link cannot be opened.
    pub fn follow(&self, document: &str, target: &str) -> Result<Place, String> {
        workspace::follow_link(&self.workspace, document, target)
    }

    /// Runs every table of a file.
    ///
    /// # Errors
    ///
    /// Fails with a message for the user when the run cannot start.
    pub fn run_file(&mut self, opened: Option<&Opened>) -> Result<(), String> {
        let opened = opened.filter(|opened| opened.is_test_file());
        let opened = opened.ok_or("This file has no temoc tests")?;
        self.start_run(opened, opened.file.clone(), opened.tables(), None)
    }

    /// Runs the table of a file with its directive on a line.
    ///
    /// # Errors
    ///
    /// Fails with a message for the user when the run cannot start.
    pub fn run_table(&mut self, chosen: Option<(&Opened, usize)>) -> Result<(), String> {
        let found = chosen.and_then(|(opened, line)| {
            let tables = opened.tables().iter();
            let mut tables = tables.filter(|table| table.directive_line == line);
            Some((opened, line, tables.next()?))
        });
        let (opened, line, table) = found.ok_or("No test table to run")?;
        let target = format!("{}:{line}", opened.file);
        self.start_run(opened, target, std::slice::from_ref(table), None)
    }

    /// Runs the row of a file on a line.
    ///
    /// # Errors
    ///
    /// Fails with a message for the user when the run cannot start.
    pub fn run_row(&mut self, chosen: Option<(&Opened, usize)>) -> Result<(), String> {
        let found = chosen.and_then(|(opened, line)| {
            let tables = opened.tables().iter();
            let mut tables = tables.filter(|table| table.rows.contains(&line));
            Some((opened, line, tables.next()?))
        });
        let (opened, line, table) = found.ok_or("No test row to run")?;
        let target = format!("{}:{line}", opened.file);
        self.start_run(opened, target, std::slice::from_ref(table), Some(line))
    }

    /// Starts temoc on `target`, which is these tables or one `row` of them.
    fn start_run(
        &mut self,
        opened: &Opened,
        target: String,
        tables: &[TestTable],
        row: Option<usize>,
    ) -> Result<(), String> {
        if self.run.is_some() {
            return Err("temoc is already running".into());
        }
        let arguments = vec![
            "-c".into(),
            self.configuration()?,
            "--output".into(),
            "jsonl".into(),
            target,
        ];
        self.temoc_arguments = Some(arguments.clone());
        let results = self.results.entry(opened.file.clone()).or_default();
        results.begin(tables, row);
        let request = Request {
            folder: self.workspace.clone(),
            extra_path: self.extra_path.clone(),
            arguments,
            file: opened.file.clone(),
            tables: tables.to_vec(),
        };
        match Run::start(request) {
            Ok(run) => {
                self.run = Some(run);
                Ok(())
            }
            Err(message) => {
                results.finish(&RunReport::stopped(message.clone()));
                Err(failed_run(&message))
            }
        }
    }

    /// The configuration file to give temoc, which runs in the workspace.
    pub(crate) fn configuration(&self) -> Result<String, String> {
        let Some(written) = &self.config else {
            return if self.workspace.join(CONFIGURATION).is_file() {
                Ok(CONFIGURATION.into())
            } else {
                Err(format!(
                    "No temoc configuration found. Add {CONFIGURATION} or use --config."
                ))
            };
        };
        let missing = || format!("Configuration file not found: {written}");
        let path = self.start_folder.join(written);
        let path = path.canonicalize().map_err(|_| missing())?;
        if !path.is_file() {
            return Err(missing());
        }
        let workspace = self.workspace.canonicalize();
        let inside = workspace.map(|workspace| path.strip_prefix(workspace).map(PathBuf::from));
        let path = match inside {
            Ok(Ok(inside)) => inside,
            _ => path,
        };
        Ok(path.to_string_lossy().into_owned())
    }

    /// Picks up the progress of the run. Says what stopped it, if anything did.
    pub fn update(&mut self) -> Option<String> {
        let run = self.run.as_mut()?;
        let Progress { events, report } = run.progress();
        let results = self.results.entry(run.file().to_owned()).or_default();
        for event in events {
            results.apply(run.tables(), event);
        }
        let report = report?;
        results.finish(&report);
        self.run = None;
        report.message.as_deref().map(failed_run)
    }

    /// Whether temoc is running.
    #[must_use]
    pub fn is_busy(&self) -> bool {
        self.run.is_some()
    }

    /// The file temoc is running.
    #[must_use]
    pub fn running_file(&self) -> Option<&str> {
        self.run.as_ref().map(Run::file)
    }

    /// The arguments of the last temoc command.
    #[must_use]
    pub fn temoc_arguments(&self) -> Option<&[String]> {
        self.temoc_arguments.as_deref()
    }

    /// What the runs of a file have reported so far.
    #[must_use]
    pub fn results(&self, file: &str) -> Option<&FileResults> {
        self.results.get(file)
    }

    /// The result for a whole file, if it has tests.
    #[must_use]
    pub fn file_result(&self, opened: &Opened) -> Option<&'static str> {
        let results = self.results(&opened.file);
        opened
            .is_test_file()
            .then(|| results.map_or(NOT_RUN, |results| results.summary(opened.tables())))
    }

    /// What is known about the tests of a file.
    #[must_use]
    pub fn outcome(&self, opened: &Opened) -> Outcome {
        let results = self.results(&opened.file).cloned().unwrap_or_default();
        Outcome {
            file: self.file_result(opened),
            tables: results.by_table(opened.tables()),
            rows: results
                .by_row()
                .into_iter()
                .map(|(line, result)| (line, result.to_string()))
                .collect(),
            failures: results.failures(opened.tables()),
        }
    }
}

/// What a run that could not finish reports.
fn failed_run(message: &str) -> String {
    format!("temoc failed: {message}")
}
