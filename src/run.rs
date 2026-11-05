//! Running temoc in the background, and keeping what it reports.

use crate::temoc::{Event, Failure, RunReport, RunResult, TestTable, Transcript};
use std::collections::{BTreeMap, HashMap};
use std::env;
use std::ffi::{OsStr, OsString};
use std::io::{self, BufRead, BufReader, ErrorKind, Read};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread::{self, JoinHandle, sleep};
use std::time::{Duration, Instant};

const TEMOC: &str = "temoc";
/// What a file without any results shows.
pub const NOT_RUN: &str = "not run";
/// How long temoc has to stop. It gives its server two seconds before killing it.
const STOP_PATIENCE: Duration = Duration::from_secs(3);
/// How often to look at a process that was asked to stop.
const STOP_POLL: Duration = Duration::from_millis(10);

/// What a row got from its latest run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowResult {
    /// The result of the row.
    pub result: RunResult,
    /// What did not pass, in the order reported.
    pub failures: Vec<Failure>,
}

/// What temoc has reported about a table.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct TableRun {
    /// Whether temoc is between the first and last rows of the table.
    running: bool,
    /// Whether temoc went through the whole table.
    finished: bool,
    /// The result of each row, by its line.
    rows: BTreeMap<usize, RowResult>,
    /// What failed outside of any row.
    failures: Vec<Failure>,
}

impl TableRun {
    /// The worst of what the table's rows got, counting a row with no result
    /// yet as `partial`. A table nothing is known about has no result.
    fn result(&self, table: &TestTable) -> Option<RunResult> {
        if self.running {
            return Some(RunResult::Running);
        }
        if self.rows.is_empty() && self.failures.is_empty() && !self.finished {
            return None;
        }
        let rows = table.rows.iter().map(|line| {
            let row = self.rows.get(line);
            row.map_or(RunResult::Partial, |row| row.result)
        });
        let failures = self.failures.iter().map(|failure| failure.result);
        Some(rows.chain(failures).max().unwrap_or(RunResult::Passed))
    }

    /// Forgets what a run that is over left unfinished.
    fn stop_running(&mut self) {
        self.running = false;
        self.rows.retain(|_, row| row.result != RunResult::Running);
    }
}

/// What a table has to show for its runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableResult<'results> {
    /// The result of the table.
    pub result: RunResult,
    /// The result of each row, by its line.
    pub rows: &'results BTreeMap<usize, RowResult>,
    /// What failed outside of any row.
    pub failures: &'results [Failure],
}

/// What the runs of a file have reported so far.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileResults {
    /// What is known about each table, by the line of its directive.
    tables: HashMap<usize, TableRun>,
    /// Whether a run of this file is going on.
    running: bool,
    /// Whether the latest run stopped without finishing.
    stopped: bool,
}

impl FileResults {
    /// The result of a table, if it has one.
    #[must_use]
    pub fn table(&self, table: &TestTable) -> Option<TableResult<'_>> {
        let run = self.tables.get(&table.directive_line)?;
        Some(TableResult {
            result: run.result(table)?,
            rows: &run.rows,
            failures: &run.failures,
        })
    }

    /// Clears what a run is about to replace: these tables, or only one row of
    /// them. Everything else stays.
    pub fn begin(&mut self, ran: &[TestTable], row: Option<usize>) {
        self.running = true;
        for table in ran {
            match (row, self.tables.get_mut(&table.directive_line)) {
                (Some(line), Some(run)) => {
                    run.rows.remove(&line);
                    run.failures.clear();
                    run.finished = false;
                }
                (Some(_), None) => {}
                (None, _) => {
                    self.tables.remove(&table.directive_line);
                }
            }
        }
    }

    /// Keeps something that happened during a run of these tables.
    pub fn apply(&mut self, ran: &[TestTable], event: Event) {
        match event {
            Event::TableStarted { line } => self.tables.entry(line).or_default().running = true,
            Event::RowStarted { table, line } => {
                let row = RowResult {
                    result: RunResult::Running,
                    failures: Vec::new(),
                };
                self.tables.entry(table).or_default().rows.insert(line, row);
            }
            Event::RowFinished {
                table,
                line,
                result,
                failures,
            } => {
                let row = RowResult { result, failures };
                self.tables.entry(table).or_default().rows.insert(line, row);
            }
            Event::TableFinished { line } => {
                let run = self.tables.entry(line).or_default();
                run.running = false;
                run.finished = true;
            }
            Event::SetupFailed(failure) => {
                if let Some(table) = owner(ran, failure.line) {
                    let run = self.tables.entry(table.directive_line).or_default();
                    run.failures.push(failure);
                }
            }
        }
    }

    /// Closes a run with the way it ended.
    pub fn finish(&mut self, report: &RunReport) {
        self.running = false;
        self.stopped = report.message.is_some();
        self.tables.values_mut().for_each(TableRun::stop_running);
    }

    /// The result for the whole file, given all of its tables: the worst of
    /// what they got, counting a table with no result yet as `partial`.
    #[must_use]
    pub fn summary(&self, tables: &[TestTable]) -> &'static str {
        if self.running {
            return RunResult::Running.name();
        }
        if self.stopped {
            return RunResult::Error.name();
        }
        let results: Vec<Option<RunResult>> = tables
            .iter()
            .map(|table| self.table(table).map(|table| table.result))
            .collect();
        if results.iter().all(Option::is_none) {
            return NOT_RUN;
        }
        let worst = results
            .iter()
            .map(|result| result.unwrap_or(RunResult::Partial))
            .max();
        worst.map_or(NOT_RUN, RunResult::name)
    }

    /// Each table's name and result, in document order.
    #[must_use]
    pub fn by_table(&self, tables: &[TestTable]) -> Vec<(String, String)> {
        tables
            .iter()
            .map(|table| {
                let result = self.table(table).map(|table| table.result.to_string());
                (table.name.clone(), result.unwrap_or_else(|| NOT_RUN.into()))
            })
            .collect()
    }

    /// Each row that has a result, by its line, in document order.
    #[must_use]
    pub fn by_row(&self) -> Vec<(usize, RunResult)> {
        let rows = self.tables.values().flat_map(|run| &run.rows);
        let rows: BTreeMap<usize, RunResult> =
            rows.map(|(line, row)| (*line, row.result)).collect();
        rows.into_iter().collect()
    }

    /// Every failure, named after its table, in document order.
    #[must_use]
    pub fn failures(&self, tables: &[TestTable]) -> Vec<String> {
        let failures = self.not_passed(tables).into_iter();
        failures
            .map(|(table, failure)| format!("{} {failure}", table.name))
            .collect()
    }

    /// Every failure with its table, in document order.
    #[must_use]
    pub fn not_passed<'tables>(
        &self,
        tables: &'tables [TestTable],
    ) -> Vec<(&'tables TestTable, &Failure)> {
        tables
            .iter()
            .filter_map(|table| Some((table, self.table(table)?)))
            .flat_map(|(table, result)| {
                let rows = result.rows.values().flat_map(|row| &row.failures);
                let failures = result.failures.iter().chain(rows);
                failures.map(move |failure| (table, failure))
            })
            .collect()
    }
}

/// The table a failure belongs to: the last one starting at or above its line.
/// A line above every table, such as a failed import, goes to the first one.
fn owner(ran: &[TestTable], line: usize) -> Option<&TestTable> {
    ran.iter()
        .rev()
        .find(|table| table.directive_line <= line)
        .or_else(|| ran.first())
}

/// A run to start.
#[derive(Debug, Clone)]
pub struct Request {
    /// The folder temoc runs in.
    pub folder: PathBuf,
    /// Folders to search ahead of `PATH`.
    pub extra_path: Vec<PathBuf>,
    /// The arguments for temoc.
    pub arguments: Vec<String>,
    /// The file that runs, relative to `folder`.
    pub file: String,
    /// The tables that run.
    pub tables: Vec<TestTable>,
}

/// What a run has done since the last look.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Progress {
    /// What happened, in order.
    pub events: Vec<Event>,
    /// How the run ended, once it has.
    pub report: Option<RunReport>,
}

/// A temoc process at work. Dropping it stops temoc.
#[derive(Debug)]
pub struct Run {
    file: String,
    tables: Vec<TestTable>,
    child: Child,
    /// The lines of temoc's output, as they arrive.
    lines: Receiver<String>,
    stderr: Option<JoinHandle<io::Result<Vec<u8>>>>,
    transcript: Transcript,
}

impl Run {
    /// Starts temoc.
    ///
    /// # Errors
    ///
    /// Fails with a message for the user when temoc cannot be started.
    pub fn start(request: Request) -> Result<Self, String> {
        Self::start_program(OsStr::new(TEMOC), request)
    }

    fn start_program(program: &OsStr, request: Request) -> Result<Self, String> {
        let describe = |error: io::Error| match error.kind() {
            ErrorKind::NotFound => format!("{} is not on your PATH", program.display()),
            _ => error.to_string(),
        };
        let mut command = Command::new(program);
        command
            .args(&request.arguments)
            .current_dir(&request.folder)
            .env("PATH", search_path(&request.extra_path).map_err(describe)?)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        // A group of its own lets temoc and what it started be stopped together.
        #[cfg(unix)]
        command.process_group(0);
        let mut child = command.spawn().map_err(describe)?;
        let (sender, lines) = mpsc::channel();
        if let Some(stdout) = child.stdout.take() {
            thread::spawn(move || {
                let mut stdout = BufReader::new(stdout);
                while let Some(line) = next_line(&mut stdout) {
                    // Nobody is listening once the run has been dropped.
                    if sender.send(line).is_err() {
                        break;
                    }
                }
            });
        }
        Ok(Self {
            file: request.file,
            tables: request.tables,
            stderr: child.stderr.take().map(read_in_background),
            transcript: Transcript::default(),
            lines,
            child,
        })
    }

    /// The file that is running, relative to the workspace.
    #[must_use]
    pub fn file(&self) -> &str {
        &self.file
    }

    /// The tables that are running.
    #[must_use]
    pub fn tables(&self) -> &[TestTable] {
        &self.tables
    }

    /// What temoc has reported since the last look.
    pub fn progress(&mut self) -> Progress {
        let mut events = Vec::new();
        loop {
            match self.lines.try_recv() {
                Ok(line) => events.extend(self.transcript.read(&line)),
                Err(TryRecvError::Empty) => {
                    let report = None;
                    return Progress { events, report };
                }
                Err(TryRecvError::Disconnected) => {
                    let report = self.finished();
                    return Progress { events, report };
                }
            }
        }
    }

    /// How the run ended, once temoc is gone. Its output has all been read.
    fn finished(&mut self) -> Option<RunReport> {
        let status = match self.child.try_wait() {
            Ok(status) => status?,
            Err(error) => return Some(RunReport::stopped(error.to_string())),
        };
        if self.stderr.as_ref().is_some_and(|pipe| !pipe.is_finished()) {
            return None;
        }
        // A process ended by a signal has no exit code.
        let exit_code = status.code().unwrap_or(-1);
        let stderr = text_of(self.stderr.take());
        Some(self.transcript.report(&stderr, exit_code))
    }
}

impl Drop for Run {
    fn drop(&mut self) {
        // The group is only certain to still be temoc's while temoc is alive.
        if !matches!(self.child.try_wait(), Ok(None)) {
            return;
        }
        stop_group(self.child.id());
        // temoc has to outlive its server to stop it, because the server is in
        // a group of its own.
        let deadline = Instant::now() + STOP_PATIENCE;
        while matches!(self.child.try_wait(), Ok(None)) && Instant::now() < deadline {
            sleep(STOP_POLL);
        }
        // Nothing more can be done about a process that will not stop.
        match self.child.kill().and_then(|()| self.child.wait()) {
            Ok(_) | Err(_) => {}
        }
    }
}

/// Asks a process group to stop, temoc included. The standard library can only
/// signal a single process.
#[cfg(unix)]
fn stop_group(leader: u32) {
    let stopped = Command::new("kill")
        .args(["-TERM", "--", &format!("-{leader}")])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    // temoc itself is stopped later either way.
    match stopped {
        Ok(_) | Err(_) => {}
    }
}

#[cfg(not(unix))]
fn stop_group(_leader: u32) {}

/// Where to look for programs, so that temoc's server is found the same way.
fn search_path(extra: &[PathBuf]) -> io::Result<OsString> {
    let path = env::var_os("PATH").unwrap_or_default();
    let folders = extra.iter().cloned().chain(env::split_paths(&path));
    env::join_paths(folders).map_err(io::Error::other)
}

/// The next line of a pipe, without its line break. Output that cannot be read
/// ends here, which reads as a run that stopped before finishing.
fn next_line(pipe: &mut impl BufRead) -> Option<String> {
    let mut line = Vec::new();
    match pipe.read_until(b'\n', &mut line) {
        Ok(1..) => Some(String::from_utf8_lossy(&line).trim_end().to_owned()),
        Ok(0) | Err(_) => None,
    }
}

/// Reads a pipe to its end. temoc would stall if a full pipe went unread.
fn read_in_background(mut pipe: impl Read + Send + 'static) -> JoinHandle<io::Result<Vec<u8>>> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        pipe.read_to_end(&mut bytes)?;
        Ok(bytes)
    })
}

/// What was read from a pipe. Output that could not be read reads as empty.
fn text_of(pipe: Option<JoinHandle<io::Result<Vec<u8>>>>) -> String {
    match pipe.map(JoinHandle::join) {
        Some(Ok(Ok(bytes))) => String::from_utf8_lossy(&bytes).into_owned(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn table(name: &str, directive_line: usize, rows: &[usize]) -> TestTable {
        TestTable {
            name: name.into(),
            directive_line,
            line: directive_line + 2,
            heading_line: None,
            rows: rows.to_vec(),
        }
    }

    fn tables() -> [TestTable; 2] {
        [table("One", 5, &[9, 10]), table("Two", 15, &[19])]
    }

    fn failure(line: usize, result: RunResult) -> Failure {
        Failure {
            line,
            message: "wrong".into(),
            snoozed_until: None,
            result,
        }
    }

    fn row(table: usize, line: usize, result: RunResult) -> Event {
        let failures = match result {
            RunResult::Failed | RunResult::Error | RunResult::Snoozed => {
                vec![failure(line, result)]
            }
            _ => Vec::new(),
        };
        Event::RowFinished {
            table,
            line,
            result,
            failures,
        }
    }

    fn passed() -> RunReport {
        RunReport {
            result: RunResult::Passed,
            message: None,
        }
    }

    /// A finished run of every table, with these rows.
    fn results_with(rows: &[(usize, usize, RunResult)]) -> FileResults {
        let mut results = FileResults::default();
        results.begin(&tables(), None);
        for (table, line, result) in rows {
            results.apply(&tables(), row(*table, *line, *result));
        }
        results.finish(&passed());
        results
    }

    fn results_of(results: &FileResults) -> Vec<String> {
        let by_table = results.by_table(&tables());
        by_table.iter().map(|(_, result)| result.clone()).collect()
    }

    const ALL_PASSED: [(usize, usize, RunResult); 3] = [
        (5, 9, RunResult::Passed),
        (5, 10, RunResult::Passed),
        (15, 19, RunResult::Passed),
    ];

    #[test]
    fn a_table_takes_the_worst_result_of_its_rows() {
        let results = results_with(&[
            (5, 9, RunResult::Snoozed),
            (5, 10, RunResult::Error),
            (15, 19, RunResult::Skipped),
        ]);
        assert_eq!(results_of(&results), ["error", "skipped"]);
        assert_eq!(results.summary(&tables()), "error");
    }

    #[test]
    fn a_row_without_a_result_makes_its_table_partial() {
        let results = results_with(&[(5, 9, RunResult::Snoozed)]);
        assert_eq!(results_of(&results), ["partial", "not run"]);
        assert_eq!(results.summary(&tables()), "partial");
    }

    #[test]
    fn a_row_that_did_not_pass_beats_partial() {
        let results = results_with(&[(5, 9, RunResult::Failed)]);
        assert_eq!(results_of(&results), ["failed", "not run"]);
        assert_eq!(results.summary(&tables()), "failed");
    }

    #[test]
    fn a_table_nothing_is_known_about_has_not_run() {
        let results = results_with(&[]);
        assert_eq!(results_of(&results), ["not run", "not run"]);
        assert_eq!(results.summary(&tables()), "not run");
    }

    #[test]
    fn a_table_without_rows_passes_once_temoc_is_done_with_it() {
        let empty = table("Empty", 3, &[]);
        let ran = std::slice::from_ref(&empty);
        let mut results = FileResults::default();
        results.begin(ran, None);
        assert_eq!(results.table(&empty), None);
        results.apply(ran, Event::TableFinished { line: 3 });
        results.finish(&passed());
        assert_eq!(
            results.table(&empty).map(|table| table.result),
            Some(RunResult::Passed)
        );
    }

    #[test]
    fn a_failure_outside_a_row_counts_for_the_table() {
        let mut results = results_with(&ALL_PASSED);
        let failure = failure(15, RunResult::Error);
        results.apply(&tables(), Event::SetupFailed(failure));
        assert_eq!(results_of(&results), ["passed", "error"]);
        assert_eq!(results.failures(&tables()), ["Two line 15: wrong"]);
    }

    #[test]
    fn a_failure_above_every_table_goes_under_the_first() {
        let mut results = results_with(&ALL_PASSED);
        let failure = failure(1, RunResult::Error);
        results.apply(&tables(), Event::SetupFailed(failure));
        assert_eq!(results_of(&results), ["error", "passed"]);
    }

    #[test]
    fn a_failure_in_a_table_run_alone_goes_to_that_table() {
        let [_, second] = tables();
        let mut results = FileResults::default();
        let ran = [second];
        results.begin(&ran, None);
        results.apply(&ran, Event::SetupFailed(failure(1, RunResult::Error)));
        results.finish(&passed());
        assert_eq!(results_of(&results), ["not run", "error"]);
    }

    #[test]
    fn failures_come_in_document_order() {
        let results = results_with(&[
            (15, 19, RunResult::Failed),
            (5, 10, RunResult::Failed),
            (5, 9, RunResult::Failed),
        ]);
        assert_eq!(
            results.failures(&tables()),
            [
                "One line 9: wrong",
                "One line 10: wrong",
                "Two line 19: wrong"
            ]
        );
    }

    #[test]
    fn rows_come_in_document_order_across_tables() {
        let results = results_with(&[(15, 19, RunResult::Passed), (5, 9, RunResult::Failed)]);
        assert_eq!(
            results.by_row(),
            [(9, RunResult::Failed), (19, RunResult::Passed)]
        );
    }

    #[test]
    fn a_table_is_running_from_its_start_to_its_finish() {
        let mut results = FileResults::default();
        results.begin(&tables(), None);
        results.apply(&tables(), Event::TableStarted { line: 5 });
        assert_eq!(results_of(&results), ["running", "not run"]);
        assert_eq!(results.summary(&tables()), "running");
        results.apply(&tables(), Event::TableFinished { line: 5 });
        assert_eq!(results_of(&results), ["partial", "not run"]);
        assert_eq!(results.summary(&tables()), "running");
    }

    #[test]
    fn a_row_is_running_until_it_finishes() {
        let mut results = FileResults::default();
        let started = Event::RowStarted { table: 5, line: 9 };
        results.apply(&tables(), started);
        assert_eq!(results.by_row(), [(9, RunResult::Running)]);
        results.apply(&tables(), row(5, 9, RunResult::Passed));
        assert_eq!(results.by_row(), [(9, RunResult::Passed)]);
    }

    #[test]
    fn a_run_that_ends_leaves_nothing_running() {
        let mut results = FileResults::default();
        results.begin(&tables(), None);
        results.apply(&tables(), Event::TableStarted { line: 5 });
        let started = Event::RowStarted { table: 5, line: 9 };
        results.apply(&tables(), started);
        results.finish(&RunReport::stopped("boom".into()));
        assert_eq!(results.by_row(), []);
        assert_eq!(results_of(&results), ["not run", "not run"]);
        assert_eq!(results.summary(&tables()), "error");
    }

    #[test]
    fn a_file_run_clears_every_table_right_away() {
        let mut results = results_with(&ALL_PASSED);
        results.begin(&tables(), None);
        assert_eq!(results_of(&results), ["not run", "not run"]);
    }

    #[test]
    fn a_table_run_keeps_the_other_tables() {
        let mut results = results_with(&ALL_PASSED);
        let [_, second] = tables();
        results.begin(&[second], None);
        assert_eq!(results_of(&results), ["passed", "not run"]);
    }

    #[test]
    fn a_row_run_keeps_the_other_rows() {
        let mut results = results_with(&ALL_PASSED);
        let [first, _] = tables();
        results.begin(&[first], Some(10));
        assert_eq!(
            results.by_row(),
            [(9, RunResult::Passed), (19, RunResult::Passed)]
        );
    }

    #[test]
    fn a_row_run_clears_the_failures_of_its_table() {
        let mut results = results_with(&[(5, 9, RunResult::Skipped)]);
        let failure = failure(5, RunResult::Error);
        results.apply(&tables(), Event::SetupFailed(failure));
        let [first, _] = tables();
        results.begin(&[first], Some(9));
        assert_eq!(results.failures(&tables()), Vec::<String>::new());
    }

    #[test]
    fn a_row_run_of_a_table_with_no_results_changes_nothing_yet() {
        let mut results = FileResults::default();
        let [first, _] = tables();
        results.begin(&[first], Some(9));
        assert_eq!(results_of(&results), ["not run", "not run"]);
    }

    #[test]
    fn a_stopped_run_stays_an_error_until_a_run_finishes() {
        let mut results = results_with(&ALL_PASSED);
        let [first, _] = tables();
        results.begin(std::slice::from_ref(&first), None);
        results.finish(&RunReport::stopped("boom".into()));
        assert_eq!(results_of(&results), ["not run", "passed"]);
        assert_eq!(results.summary(&tables()), "error");
        results.begin(&[first], None);
        results.finish(&passed());
        assert_ne!(results.summary(&tables()), "error");
    }

    fn request(arguments: &[&str]) -> Request {
        Request {
            folder: env::temp_dir(),
            extra_path: Vec::new(),
            arguments: arguments.iter().map(ToString::to_string).collect(),
            file: "a.md".into(),
            tables: Vec::new(),
        }
    }

    fn start(script: &str) -> Run {
        Run::start_program(OsStr::new("sh"), request(&["-c", script])).unwrap()
    }

    /// Everything a run reports, up to its end.
    fn wait_for(run: &mut Run) -> (Vec<Event>, RunReport) {
        let mut events = Vec::new();
        loop {
            let progress = run.progress();
            events.extend(progress.events);
            if let Some(report) = progress.report {
                return (events, report);
            }
            sleep(Duration::from_millis(5));
        }
    }

    fn file_in_temp(name: &str) -> PathBuf {
        env::temp_dir().join(format!("temoc-ui-{name}-{}", std::process::id()))
    }

    fn wait_for_file(file: &PathBuf) -> String {
        loop {
            match fs::read_to_string(file) {
                Ok(text) if text.ends_with('\n') => {
                    fs::remove_file(file).unwrap();
                    return text.trim().to_owned();
                }
                _ => sleep(Duration::from_millis(5)),
            }
        }
    }

    const STARTED: &str = r#"echo '{"type":"run_started","schema":1}'"#;
    const FINISHED: &str = r#"echo '{"type":"run_finished","status":"passed"}'"#;

    #[test]
    fn a_missing_program_says_so() {
        let result = Run::start_program(OsStr::new("no-such-program"), request(&[]));
        assert_eq!(
            result.err().as_deref(),
            Some("no-such-program is not on your PATH")
        );
    }

    #[test]
    fn the_end_of_the_run_comes_from_its_output() {
        let mut run = start(&format!("{STARTED}; {FINISHED}"));
        assert_eq!(wait_for(&mut run).1.result, RunResult::Passed);
    }

    #[test]
    fn events_arrive_while_the_program_is_still_working() {
        let file = file_in_temp("go-on");
        let script = format!(
            r#"{STARTED}; echo '{{"type":"table_started","line":7}}'; \
               until [ -e {file} ]; do sleep 0.01; done; {FINISHED}"#,
            file = file.display()
        );
        let mut run = start(&script);
        let mut events = Vec::new();
        while events.is_empty() {
            let progress = run.progress();
            assert_eq!(progress.report, None);
            events = progress.events;
            sleep(Duration::from_millis(5));
        }
        assert_eq!(events, [Event::TableStarted { line: 7 }]);
        fs::write(&file, "").unwrap();
        assert_eq!(wait_for(&mut run).1.result, RunResult::Passed);
        fs::remove_file(&file).unwrap();
    }

    #[test]
    fn more_output_than_a_pipe_holds_does_not_stall_the_program() {
        let mut run = start(&format!(
            "{STARTED}; head -c 1000000 /dev/zero >&2; {FINISHED}"
        ));
        assert_eq!(wait_for(&mut run).1.result, RunResult::Passed);
    }

    #[test]
    fn a_program_ended_by_a_signal_stopped_before_finishing() {
        let mut run = start(&format!("{STARTED}; kill -9 $$"));
        assert_eq!(
            wait_for(&mut run).1.message.as_deref(),
            Some("temoc stopped before finishing")
        );
    }

    #[test]
    fn stderr_explains_a_run_that_printed_nothing() {
        let mut run = start("echo 'no such option' >&2; exit 2");
        assert_eq!(
            wait_for(&mut run).1.message.as_deref(),
            Some("temoc-ui needs temoc 0.10 or later: no such option")
        );
    }

    #[test]
    fn a_program_still_working_has_not_finished() {
        let mut run = Run::start_program(OsStr::new("sleep"), request(&["30"])).unwrap();
        assert_eq!(run.progress().report, None);
    }

    fn is_alive(pid: &str) -> bool {
        let status = Command::new("kill")
            .args(["-0", pid])
            .stderr(Stdio::null())
            .status();
        status.unwrap().success()
    }

    #[test]
    fn dropping_a_run_stops_what_the_program_started() {
        let file = file_in_temp("server");
        let script = format!("sleep 30 & echo $! > {}; wait", file.display());
        let run = start(&script);
        let server = wait_for_file(&file);
        assert!(is_alive(&server));
        drop(run);
        // A stopped process lingers until the system collects it.
        for _ in 0..200 {
            if !is_alive(&server) {
                return;
            }
            sleep(Duration::from_millis(5));
        }
        panic!("the server is still running");
    }

    #[test]
    fn dropping_a_run_stops_the_program() {
        let run = Run::start_program(OsStr::new("sleep"), request(&["30"])).unwrap();
        let pid = run.child.id().to_string();
        drop(run);
        assert!(!is_alive(&pid));
    }

    /// temoc stops its server after it is asked to stop. Killing temoc at once
    /// would leave the server behind.
    #[test]
    fn dropping_a_run_gives_the_program_time_to_clean_up() {
        let ready = file_in_temp("ready");
        let cleaned = file_in_temp("cleaned");
        let script = format!(
            "trap 'sleep 0.2; echo done > {cleaned}; exit 0' TERM; \
             echo ready > {ready}; while :; do sleep 0.01; done",
            cleaned = cleaned.display(),
            ready = ready.display()
        );
        let run = start(&script);
        wait_for_file(&ready);
        drop(run);
        assert_eq!(fs::read_to_string(&cleaned).unwrap(), "done\n");
        fs::remove_file(&cleaned).unwrap();
    }
}
