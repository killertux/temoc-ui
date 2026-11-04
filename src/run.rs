//! Running temoc in the background, and keeping what it reports.

use crate::temoc::{self, Failure, RunReport, RunResult, TestTable};
use std::collections::HashMap;
use std::env;
use std::ffi::{OsStr, OsString};
use std::io::{self, ErrorKind, Read};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread::{self, JoinHandle};

const TEMOC: &str = "temoc";

/// What a table got from its latest run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableResult {
    /// The result of the table.
    pub result: RunResult,
    /// Its failures, in the order reported.
    pub failures: Vec<Failure>,
}

impl TableResult {
    fn passed() -> Self {
        Self {
            result: RunResult::Passed,
            failures: Vec::new(),
        }
    }

    fn add(&mut self, failure: Failure) {
        self.result = match (self.result, &failure.snoozed_until) {
            (RunResult::Failed, _) | (_, None) => RunResult::Failed,
            (_, Some(_)) => RunResult::Snoozed,
        };
        self.failures.push(failure);
    }
}

/// What the runs of a file have reported so far.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileResults {
    /// The result of each table, by the line of its directive.
    tables: HashMap<usize, TableResult>,
    /// Whether the latest run stopped without a result.
    stopped: bool,
}

impl FileResults {
    /// The result of a table, if it has one.
    #[must_use]
    pub fn table(&self, table: &TestTable) -> Option<&TableResult> {
        self.tables.get(&table.directive_line)
    }

    /// Keeps what a run of these tables reported. Other tables stay as they are.
    pub fn record(&mut self, ran: &[TestTable], report: RunReport) {
        if report.result == RunResult::NoTests {
            return;
        }
        self.stopped = report.result == RunResult::Error;
        for table in ran {
            self.tables.remove(&table.directive_line);
            if !self.stopped {
                self.tables
                    .insert(table.directive_line, TableResult::passed());
            }
        }
        if self.stopped {
            return;
        }
        for failure in report.failures {
            let table = owner(ran, failure.line);
            if let Some(result) = table.and_then(|table| self.tables.get_mut(&table.directive_line))
            {
                result.add(failure);
            }
        }
    }

    /// The result for the whole file, given all of its tables.
    #[must_use]
    pub fn summary(&self, tables: &[TestTable]) -> &'static str {
        let results: Vec<Option<RunResult>> = tables
            .iter()
            .map(|table| self.table(table).map(|table| table.result))
            .collect();
        let has = |result| results.contains(&Some(result));
        if self.stopped {
            "error"
        } else if results.iter().all(Option::is_none) {
            "not run"
        } else if has(RunResult::Failed) {
            "failed"
        } else if results.contains(&None) {
            "partial"
        } else if has(RunResult::Snoozed) {
            "snoozed"
        } else {
            "passed"
        }
    }

    /// Each table's name and result, in document order.
    #[must_use]
    pub fn by_table(&self, tables: &[TestTable]) -> Vec<(String, String)> {
        tables
            .iter()
            .map(|table| {
                let result = self.table(table).map(|table| table.result.to_string());
                (
                    table.name.clone(),
                    result.unwrap_or_else(|| "not run".into()),
                )
            })
            .collect()
    }

    /// Every failure, named after its table, in document order.
    #[must_use]
    pub fn failures(&self, tables: &[TestTable]) -> Vec<String> {
        tables
            .iter()
            .flat_map(|table| {
                let failures = self.table(table).map_or(&[][..], |t| t.failures.as_slice());
                failures
                    .iter()
                    .map(|failure| format!("{} {failure}", table.name))
            })
            .collect()
    }
}

/// The table a failure belongs to: the last one starting at or above its line.
/// With no line, that is the last table. A line above every table, such as a
/// failed import, goes to the first one.
fn owner(ran: &[TestTable], line: Option<usize>) -> Option<&TestTable> {
    let line = line.unwrap_or(usize::MAX);
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

/// A temoc process at work. Dropping it stops temoc.
#[derive(Debug)]
pub struct Run {
    file: String,
    tables: Vec<TestTable>,
    child: Child,
    stdout: Option<JoinHandle<io::Result<Vec<u8>>>>,
    stderr: Option<JoinHandle<io::Result<Vec<u8>>>>,
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
        // A group of its own lets temoc and its server be stopped together.
        #[cfg(unix)]
        command.process_group(0);
        let mut child = command.spawn().map_err(describe)?;
        Ok(Self {
            file: request.file,
            tables: request.tables,
            stdout: child.stdout.take().map(read_in_background),
            stderr: child.stderr.take().map(read_in_background),
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

    /// What temoc reported, once it is done.
    pub fn finished(&mut self) -> Option<RunReport> {
        let status = match self.child.try_wait() {
            Ok(status) => status?,
            Err(error) => return Some(RunReport::stopped(error.to_string())),
        };
        let reading = |pipe: &Option<JoinHandle<_>>| {
            pipe.as_ref().is_some_and(|handle| !handle.is_finished())
        };
        if reading(&self.stdout) || reading(&self.stderr) {
            return None;
        }
        let (stdout, stderr) = (text_of(self.stdout.take()), text_of(self.stderr.take()));
        // A process ended by a signal has no exit code.
        let exit_code = status.code().unwrap_or(-1);
        Some(temoc::read_output(&stdout, &stderr, exit_code))
    }
}

impl Drop for Run {
    fn drop(&mut self) {
        // The group is only certain to still be temoc's while temoc is alive.
        if matches!(self.child.try_wait(), Ok(None)) {
            stop_group(self.child.id());
            // Nothing more can be done about a process that will not stop.
            match self.child.kill().and_then(|()| self.child.wait()) {
                Ok(_) | Err(_) => {}
            }
        }
    }
}

/// Asks a process group to stop, so that temoc's server stops with it. The
/// standard library can only signal a single process.
#[cfg(unix)]
fn stop_group(leader: u32) {
    let stopped = Command::new("kill")
        .args(["-TERM", "--", &format!("-{leader}")])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    // temoc itself is stopped next either way.
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

/// Reads a pipe to its end. temoc would stall if a full pipe went unread.
fn read_in_background(mut pipe: impl Read + Send + 'static) -> JoinHandle<io::Result<Vec<u8>>> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        pipe.read_to_end(&mut bytes)?;
        Ok(bytes)
    })
}

/// What was read from a pipe. Output that could not be read reads as empty,
/// which [`temoc::read_output`] reports as an error.
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
    use std::thread::sleep;
    use std::time::Duration;

    fn table(name: &str, directive_line: usize) -> TestTable {
        TestTable {
            name: name.into(),
            directive_line,
            line: directive_line + 2,
            heading_line: None,
        }
    }

    fn tables() -> [TestTable; 2] {
        [table("One", 5), table("Two", 15)]
    }

    fn report(stdout: &str, exit_code: i32) -> RunReport {
        temoc::read_output(&format!("Testing file a.md...{stdout}"), "", exit_code)
    }

    fn results_of(results: &FileResults) -> Vec<String> {
        let by_table = results.by_table(&tables());
        by_table.iter().map(|(_, result)| result.clone()).collect()
    }

    const FAILURE: &str = "FAIL\nExpected `5`, got `4` in a.md:20:11 for method call mul";

    #[test]
    fn a_failure_belongs_to_the_table_holding_its_line() {
        let mut results = FileResults::default();
        results.record(&tables(), report(FAILURE, 1));
        assert_eq!(results_of(&results), ["passed", "failed"]);
    }

    #[test]
    fn a_failure_without_a_line_goes_under_the_last_table() {
        let mut results = FileResults::default();
        results.record(&tables(), report("FAIL\nDifferent ID in response", 1));
        assert_eq!(results_of(&results), ["passed", "failed"]);
    }

    #[test]
    fn a_failure_above_every_table_goes_under_the_first() {
        let line = "FAIL\nExpected OK, got Exception `no import` in a.md:3:1";
        let mut results = FileResults::default();
        results.record(&tables(), report(line, 1));
        assert_eq!(results_of(&results), ["failed", "passed"]);
    }

    #[test]
    fn a_real_failure_is_not_hidden_by_a_later_snoozed_one() {
        let lines = "SNOOZED\nExpected `9`, got `3` in a.md:16:1\n\
                     Expected `5`, got `4` in a.md:17:1 -- snoozed until 2099-12-31";
        let mut results = FileResults::default();
        results.record(&tables(), report(lines, 1));
        assert_eq!(results_of(&results), ["passed", "failed"]);
    }

    #[test]
    fn running_a_file_again_replaces_old_failures() {
        let mut results = FileResults::default();
        results.record(&tables(), report(FAILURE, 1));
        results.record(&tables(), report("OK", 0));
        assert_eq!(results.failures(&tables()), Vec::<String>::new());
        assert_eq!(results.summary(&tables()), "passed");
    }

    #[test]
    fn a_stopped_run_clears_only_the_tables_it_ran() {
        let mut results = FileResults::default();
        results.record(&tables(), report("OK", 0));
        let [first, _] = tables();
        results.record(&[first], RunReport::stopped("boom".into()));
        assert_eq!(results_of(&results), ["not run", "passed"]);
        assert_eq!(results.summary(&tables()), "error");
    }

    #[test]
    fn a_finished_run_ends_the_error() {
        let mut results = FileResults::default();
        results.record(&tables(), RunReport::stopped("boom".into()));
        results.record(&tables(), report("OK", 0));
        assert_eq!(results.summary(&tables()), "passed");
    }

    #[test]
    fn a_run_that_found_nothing_changes_nothing() {
        let mut results = FileResults::default();
        results.record(&tables(), report("OK", 0));
        results.record(&tables(), report("NONE", 0));
        assert_eq!(results_of(&results), ["passed", "passed"]);
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

    fn wait_for(run: &mut Run) -> RunReport {
        loop {
            if let Some(report) = run.finished() {
                return report;
            }
            sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn a_missing_program_says_so() {
        let result = Run::start_program(OsStr::new("no-such-program"), request(&[]));
        assert_eq!(
            result.err().as_deref(),
            Some("no-such-program is not on your PATH")
        );
    }

    #[test]
    fn output_is_read_when_the_program_finishes() {
        let script = "echo 'Testing file a.md...FAIL'; echo oops; echo 'Error: bad' >&2; exit 1";
        let mut run = Run::start_program(OsStr::new("sh"), request(&["-c", script])).unwrap();
        let report = wait_for(&mut run);
        assert_eq!(report.result, RunResult::Failed);
        assert_eq!(report.failures.len(), 1);
    }

    #[test]
    fn more_output_than_a_pipe_holds_does_not_stall_the_program() {
        let script = "echo 'Testing file a.md...OK'; head -c 1000000 /dev/zero >&2";
        let mut run = Run::start_program(OsStr::new("sh"), request(&["-c", script])).unwrap();
        assert_eq!(wait_for(&mut run).result, RunResult::Passed);
    }

    #[test]
    fn a_program_ended_by_a_signal_is_an_error() {
        let script = "echo 'Testing file a.md...OK'; kill -9 $$";
        let mut run = Run::start_program(OsStr::new("sh"), request(&["-c", script])).unwrap();
        assert_eq!(wait_for(&mut run).result, RunResult::Error);
    }

    #[test]
    fn a_program_still_working_has_not_finished() {
        let mut run = Run::start_program(OsStr::new("sleep"), request(&["30"])).unwrap();
        assert_eq!(run.finished(), None);
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
        let file = env::temp_dir().join(format!("temoc-ui-server-{}", std::process::id()));
        let script = format!("sleep 30 & echo $! > {}; wait", file.display());
        let run = Run::start_program(OsStr::new("sh"), request(&["-c", &script])).unwrap();
        let server = loop {
            match fs::read_to_string(&file) {
                Ok(pid) if pid.ends_with('\n') => break pid.trim().to_owned(),
                _ => sleep(Duration::from_millis(5)),
            }
        };
        fs::remove_file(&file).unwrap();
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
        let alive = Command::new("kill")
            .args(["-0", &pid])
            .stderr(Stdio::null())
            .status()
            .unwrap();
        assert!(!alive.success());
    }
}
