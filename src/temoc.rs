//! Understanding temoc: its test files and its output.

use markdown::ParseOptions;
use markdown::mdast::{Definition, Node};
use serde::Deserialize;
use std::fmt::{self, Display, Formatter};
use std::path::Path;
use std::{error, fs, io, slice};

/// The version of temoc's output that temoc-ui reads.
const SCHEMA: u32 = 1;

/// What temoc would find in a markdown file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TestFile {
    /// Whether the file has at least one `decisionTable` directive.
    pub is_test_file: bool,
    /// Each test table that can run, in document order.
    pub tables: Vec<TestTable>,
}

/// A test table that can run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestTable {
    /// The nearest heading above it, or else its fixture class.
    pub name: String,
    /// The line of its `decisionTable` directive, which is how temoc selects it.
    pub directive_line: usize,
    /// The line of the markdown the table starts on.
    pub line: usize,
    /// The line of the nearest heading above it.
    pub heading_line: Option<usize>,
    /// The line of each row below the header, which is how temoc selects it.
    pub rows: Vec<usize>,
}

/// Why a markdown file could not be inspected.
#[derive(Debug)]
pub enum InspectError {
    /// The file could not be read.
    Read(io::Error),
    /// The file could not be parsed as markdown.
    Parse(String),
}

impl Display for InspectError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(error) => error.fmt(f),
            Self::Parse(message) => f.write_str(message),
        }
    }
}

impl error::Error for InspectError {}

/// Looks for temoc tests in a markdown file.
///
/// # Errors
///
/// Fails when the file cannot be read or parsed.
pub fn inspect_file(path: &Path) -> Result<TestFile, InspectError> {
    find_tests(&fs::read_to_string(path).map_err(InspectError::Read)?)
}

/// Looks for temoc tests in markdown, by the same rules as temoc: only
/// top-level directives count, and a directive's table must come right after it.
///
/// # Errors
///
/// Fails when the markdown cannot be parsed.
pub fn find_tests(markdown: &str) -> Result<TestFile, InspectError> {
    let root = markdown::to_mdast(markdown, &ParseOptions::gfm())
        .map_err(|message| InspectError::Parse(message.to_string()))?;
    Ok(tests_in(&root))
}

/// Looks for temoc tests in parsed markdown. See [`find_tests`].
#[must_use]
pub fn tests_in(root: &Node) -> TestFile {
    let mut found = TestFile::default();
    let mut heading = None;
    let mut nodes = root.children().into_iter().flatten().peekable();
    while let Some(node) = nodes.next() {
        match node {
            Node::Heading(_) => heading = Some((node.to_string(), line_of(node))),
            Node::Definition(definition) => {
                let Some(class) = directive_class(definition) else {
                    continue;
                };
                found.is_test_file = true;
                if let Some(table @ Node::Table(_)) = nodes.peek() {
                    found.tables.push(TestTable {
                        name: heading
                            .as_ref()
                            .map_or_else(|| class.to_owned(), |(name, _)| name.clone()),
                        directive_line: line_of(node),
                        line: line_of(table),
                        heading_line: heading.as_ref().map(|(_, line)| *line),
                        rows: row_lines(table),
                    });
                }
            }
            _ => {}
        }
    }
    found
}

/// The lines of a table's rows, leaving out the header.
fn row_lines(table: &Node) -> Vec<usize> {
    let rows = table.children().into_iter().flatten();
    rows.skip(1).map(line_of).collect()
}

fn line_of(node: &Node) -> usize {
    node.position().map_or(0, |position| position.start.line)
}

/// The fixture class of a `decisionTable` directive, without the snooze or
/// method that may follow it.
fn directive_class(definition: &Definition) -> Option<&str> {
    if definition.url != "#" || definition.identifier != "//" {
        return None;
    }
    let (command, class) = definition.title.as_deref()?.split_once(' ')?;
    if command != "decisionTable" {
        return None;
    }
    let class = class.trim();
    let class = class.split_once(" -- ").map_or(class, |(class, _)| class);
    Some(class.split_once('#').map_or(class, |(class, _)| class))
}

/// The result of a row, a table, a file or a run. Results come in order from
/// best to worst, so that the worst of several is the greatest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RunResult {
    /// Everything matched.
    Passed,
    /// It never ran.
    Skipped,
    /// Whatever did not pass is under a snooze that is still active.
    Snoozed,
    /// Only some of it has a result.
    Partial,
    /// A wrong value.
    Failed,
    /// Anything else that stopped it from giving an answer.
    Error,
    /// temoc is working on it.
    Running,
}

impl RunResult {
    /// The word for this result.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Skipped => "skipped",
            Self::Snoozed => "snoozed",
            Self::Partial => "partial",
            Self::Failed => "failed",
            Self::Error => "error",
            Self::Running => "running",
        }
    }

    /// The mark that stands for this result.
    #[must_use]
    pub fn mark(self) -> &'static str {
        match self {
            Self::Passed => "✓",
            Self::Skipped => "-",
            Self::Snoozed => "~",
            Self::Partial => "·",
            Self::Failed => "✗",
            Self::Error => "!",
            Self::Running => "⋯",
        }
    }
}

impl Display for RunResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// One thing that did not pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    /// The line of its row, or else of what failed to set up.
    pub line: usize,
    /// What went wrong.
    pub message: String,
    /// The date a snoozed failure is snoozed until.
    pub snoozed_until: Option<String>,
    /// How much it counts for.
    pub result: RunResult,
}

impl Failure {
    /// What went wrong, without saying where.
    #[must_use]
    pub fn note(&self) -> String {
        match &self.snoozed_until {
            Some(date) => format!("{} (snoozed until {date})", self.message),
            None => self.message.clone(),
        }
    }
}

impl Display for Failure {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.note())
    }
}

/// Something that happened during a run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// temoc started on the table at the line of this directive.
    TableStarted {
        /// The line of its directive.
        line: usize,
    },
    /// temoc started on a row.
    RowStarted {
        /// The line of its table's directive.
        table: usize,
        /// The line of the row.
        line: usize,
    },
    /// A row has its result.
    RowFinished {
        /// The line of its table's directive.
        table: usize,
        /// The line of the row.
        line: usize,
        /// The result of the row.
        result: RunResult,
        /// What did not pass.
        failures: Vec<Failure>,
    },
    /// temoc is done with a table.
    TableFinished {
        /// The line of its directive.
        line: usize,
    },
    /// Something failed outside of any row.
    SetupFailed(Failure),
}

impl Event {
    /// What this event says did not pass.
    #[must_use]
    pub fn failures(&self) -> &[Failure] {
        match self {
            Self::RowFinished { failures, .. } => failures,
            Self::SetupFailed(failure) => slice::from_ref(failure),
            _ => &[],
        }
    }
}

/// A line of temoc's output. temoc may add event types and fields at any time,
/// so that anything not listed here is left out.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Line {
    RunStarted {
        schema: u32,
    },
    TableStarted {
        line: usize,
    },
    RowStarted {
        table: usize,
        line: usize,
    },
    RowFinished {
        table: usize,
        line: usize,
        status: Status,
        #[serde(default)]
        checks: Vec<Check>,
        message: Option<String>,
    },
    TableFinished {
        line: usize,
    },
    SetupFailed {
        line: usize,
        phase: String,
        status: Status,
        actual: Option<Actual>,
        snoozed_until: Option<String>,
    },
    RunError {
        message: String,
    },
    RunFinished {
        status: Status,
    },
    #[serde(other)]
    Other,
}

/// A status, as temoc writes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Status {
    Passed,
    Failed,
    Error,
    Snoozed,
    Skipped,
    Cancelled,
}

impl Status {
    fn result(self) -> RunResult {
        match self {
            Self::Passed => RunResult::Passed,
            Self::Failed => RunResult::Failed,
            Self::Snoozed => RunResult::Snoozed,
            Self::Skipped => RunResult::Skipped,
            Self::Error | Self::Cancelled => RunResult::Error,
        }
    }

    /// Whether something with this status needs explaining.
    fn did_not_pass(self) -> bool {
        !matches!(self, Self::Passed | Self::Skipped)
    }
}

/// One cell of a row, or a step around it.
#[derive(Debug, Deserialize)]
struct Check {
    method: String,
    status: Status,
    expected: Option<String>,
    actual: Option<Actual>,
    snoozed_until: Option<String>,
}

/// What the server answered.
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Actual {
    Value {
        value: String,
    },
    Void,
    List {
        value: Vec<Actual>,
    },
    Exception {
        message: String,
    },
    MissingSymbol {
        symbol: String,
    },
    ProtocolError {
        message: String,
    },
    #[serde(other)]
    Other,
}

impl Display for Actual {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Value { value } => f.write_str(first_line(value)),
            Self::Void => f.write_str("no value"),
            Self::List { value } => {
                let items: Vec<String> = value.iter().map(ToString::to_string).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Self::Exception { message } => write!(f, "exception {}", first_line(message)),
            Self::MissingSymbol { symbol } => write!(f, "unknown symbol ${symbol}"),
            Self::ProtocolError { message } => write!(f, "protocol error {}", first_line(message)),
            Self::Other => f.write_str("an answer temoc-ui does not know"),
        }
    }
}

/// The first line that has anything on it.
fn first_line(text: &str) -> &str {
    let mut lines = text.lines().map(str::trim);
    lines.find(|line| !line.is_empty()).unwrap_or_default()
}

/// Says what was expected, if anything was, and what came back.
fn describe(subject: &str, expected: Option<&str>, actual: Option<&Actual>) -> String {
    match (expected.map(first_line), actual) {
        (Some(expected), Some(actual)) => format!("{subject} expected {expected}, got {actual}"),
        (Some(expected), None) => format!("{subject} expected {expected}"),
        (None, Some(actual)) => format!("{subject} got {actual}"),
        (None, None) => subject.to_owned(),
    }
}

/// What did not pass in a row, in the order temoc reported it.
fn row_failures(line: usize, checks: &[Check], message: Option<&str>, row: Status) -> Vec<Failure> {
    let checks = checks.iter().filter(|check| check.status.did_not_pass());
    let mut failures: Vec<Failure> = checks
        .map(|check| {
            let message = describe(
                &check.method,
                check.expected.as_deref(),
                check.actual.as_ref(),
            );
            failure(line, message, check.status, check.snoozed_until.as_ref())
        })
        .collect();
    failures.extend(message.map(|message| failure(line, first_line(message).into(), row, None)));
    failures
}

/// A failure. A date only counts for a snooze that is still active, because
/// temoc keeps the date of an expired one on a normal failure.
fn failure(line: usize, message: String, status: Status, date: Option<&String>) -> Failure {
    Failure {
        line,
        message,
        snoozed_until: date.filter(|_| status == Status::Snoozed).cloned(),
        result: status.result(),
    }
}

/// How a temoc run ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunReport {
    /// The overall result.
    pub result: RunResult,
    /// What stopped the run from finishing, if anything did.
    pub message: Option<String>,
}

impl RunReport {
    /// A run that stopped without a result, with the reason why.
    #[must_use]
    pub fn stopped(message: String) -> Self {
        Self {
            result: RunResult::Error,
            message: Some(message),
        }
    }
}

/// Follows temoc's output line by line, and works out how the run ended.
#[derive(Debug, Clone, Default)]
pub struct Transcript {
    started: bool,
    finished: Option<Status>,
    /// The first problem temoc reported with the run.
    run_error: Option<String>,
    /// Why the rest of the output cannot be trusted.
    problem: Option<String>,
}

impl Transcript {
    /// Reads one line of output, and gives the event it holds, if any.
    pub fn read(&mut self, text: &str) -> Option<Event> {
        if self.problem.is_some() || text.trim().is_empty() {
            return None;
        }
        let Ok(line) = serde_json::from_str(text) else {
            self.problem = Some(format!(
                "temoc printed something temoc-ui cannot read: {text}"
            ));
            return None;
        };
        self.follow(line)
    }

    fn follow(&mut self, line: Line) -> Option<Event> {
        match line {
            Line::RunStarted { schema } => {
                self.started = true;
                if schema != SCHEMA {
                    self.problem = Some(format!(
                        "temoc-ui reads schema {SCHEMA} of temoc's output, \
                         but this temoc uses schema {schema}"
                    ));
                }
                None
            }
            Line::TableStarted { line } => Some(Event::TableStarted { line }),
            Line::RowStarted { table, line } => Some(Event::RowStarted { table, line }),
            Line::RowFinished {
                table,
                line,
                status,
                checks,
                message,
            } => Some(Event::RowFinished {
                table,
                line,
                result: status.result(),
                failures: row_failures(line, &checks, message.as_deref(), status),
            }),
            Line::TableFinished { line } => Some(Event::TableFinished { line }),
            Line::SetupFailed {
                line,
                phase,
                status,
                actual,
                snoozed_until,
            } => {
                let message = describe(&phase, None, actual.as_ref());
                let failure = failure(line, message, status, snoozed_until.as_ref());
                Some(Event::SetupFailed(failure))
            }
            Line::RunError { message } => {
                self.run_error.get_or_insert(message);
                None
            }
            Line::RunFinished { status } => {
                self.finished = Some(status);
                None
            }
            Line::Other => None,
        }
    }

    /// How the run ended, once temoc has stopped with this exit code and left
    /// this on stderr. Anything short of a `run_finished` that can be read is
    /// an error. A temoc that printed nothing and failed did not know
    /// `--output`, because any other failure comes with events.
    #[must_use]
    pub fn report(&self, stderr: &str, exit_code: i32) -> RunReport {
        if let Some(problem) = &self.problem {
            return RunReport::stopped(problem.clone());
        }
        match self.finished {
            Some(Status::Cancelled) => RunReport::stopped("temoc was stopped".into()),
            Some(status) => RunReport {
                result: status.result(),
                message: self.run_error.clone(),
            },
            None if self.started || exit_code == 0 => {
                RunReport::stopped(with_stderr("temoc stopped before finishing", stderr))
            }
            None => RunReport::stopped(with_stderr("temoc-ui needs temoc 0.10 or later", stderr)),
        }
    }
}

fn with_stderr(message: &str, stderr: &str) -> String {
    match first_line(stderr) {
        "" => message.to_owned(),
        line => format!("{message}: {line}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tables(markdown: &str) -> Vec<String> {
        let tables = find_tests(markdown).unwrap().tables;
        tables.into_iter().map(|table| table.name).collect()
    }

    #[test]
    fn a_snooze_or_method_is_not_part_of_the_fixture_class() {
        let markdown = "\
[//]: # \"decisionTable Calculator -- snooze until 2099-12-31\"

| a |
| - |
| 1 |

[//]: # \"decisionTable Calculator#sum\"

| a |
| - |
| 1 |
";
        assert_eq!(tables(markdown), ["Calculator", "Calculator"]);
    }

    #[test]
    fn a_table_after_something_else_is_not_the_directives_table() {
        let markdown = "\
[//]: # \"decisionTable Calculator\"

Not the table.

| a |
| - |
| 1 |
";
        let found = find_tests(markdown).unwrap();
        assert!(found.is_test_file);
        assert_eq!(found.tables, []);
    }

    #[test]
    fn only_comment_definitions_are_directives() {
        let found = find_tests("[note]: # \"decisionTable Calculator\"\n").unwrap();
        assert!(!found.is_test_file);
        let found = find_tests("[//]: /other \"decisionTable Calculator\"\n").unwrap();
        assert!(!found.is_test_file);
    }

    #[test]
    fn the_nearest_heading_names_a_table_as_plain_text() {
        let markdown = "\
# First

## The `sum` *method*

> ## Quoted

[//]: # \"decisionTable Calculator\"

| a |
| - |
| 1 |
";
        assert_eq!(tables(markdown), ["The sum method"]);
    }

    #[test]
    fn a_table_knows_where_it_and_its_heading_are() {
        let markdown = "# A\n\n[//]: # \"decisionTable B\"\n\n| a |\n| - |\n| 1 |\n";
        assert_eq!(
            find_tests(markdown).unwrap().tables,
            [TestTable {
                name: "A".into(),
                directive_line: 3,
                line: 5,
                heading_line: Some(1),
                rows: vec![7],
            }]
        );
    }

    #[test]
    fn a_file_that_cannot_be_read_is_an_error() {
        let result = inspect_file(Path::new("/no/such/file.md"));
        assert!(matches!(result, Err(InspectError::Read(_))));
    }

    #[test]
    fn a_table_knows_its_rows_but_not_its_header() {
        let markdown = "[//]: # \"decisionTable B\"\n\n| a |\n| - |\n| 1 |\n| 2 |\n\n";
        let tables = find_tests(markdown).unwrap().tables;
        assert_eq!(
            tables.first().map(|table| table.rows.clone()),
            Some(vec![5, 6])
        );
    }

    #[test]
    fn results_are_ordered_from_best_to_worst() {
        let results = [
            RunResult::Passed,
            RunResult::Skipped,
            RunResult::Snoozed,
            RunResult::Partial,
            RunResult::Failed,
            RunResult::Error,
        ];
        assert!(results.is_sorted());
    }

    fn events(lines: &[&str]) -> Vec<Event> {
        let mut transcript = Transcript::default();
        lines
            .iter()
            .filter_map(|line| transcript.read(line))
            .collect()
    }

    fn report(lines: &[&str], stderr: &str) -> RunReport {
        let mut transcript = Transcript::default();
        for line in lines {
            transcript.read(line);
        }
        transcript.report(stderr, 0)
    }

    const STARTED: &str = r#"{"type":"run_started","schema":1}"#;
    const FINISHED: &str = r#"{"type":"run_finished","status":"passed"}"#;

    #[test]
    fn a_row_is_followed_from_start_to_finish() {
        let lines = [
            r#"{"type":"table_started","file":"a.md","line":7}"#,
            r#"{"type":"row_started","file":"a.md","table":7,"line":11}"#,
            r#"{"type":"row_finished","table":7,"line":11,"status":"passed","checks":[]}"#,
            r#"{"type":"table_finished","line":7,"status":"passed"}"#,
        ];
        assert_eq!(
            events(&lines),
            [
                Event::TableStarted { line: 7 },
                Event::RowStarted { table: 7, line: 11 },
                Event::RowFinished {
                    table: 7,
                    line: 11,
                    result: RunResult::Passed,
                    failures: Vec::new(),
                },
                Event::TableFinished { line: 7 },
            ]
        );
    }

    #[test]
    fn a_list_inside_a_list_is_shown_in_full() {
        let line = r#"{"type":"row_finished","table":1,"line":2,"status":"failed","checks":[{"method":"m","status":"failed","expected":"x","actual":{"kind":"list","value":[{"kind":"list","value":[{"kind":"value","value":"a"}]},{"kind":"exception","message":"b"}]}}]}"#;
        let found = events(&[line]);
        let failures = found.first().map_or(&[][..], Event::failures);
        assert_eq!(
            failures.first().map(Failure::note).as_deref(),
            Some("m expected x, got [[a], exception b]")
        );
    }

    #[test]
    fn an_answer_of_an_unknown_kind_does_not_lose_the_row() {
        let line = r#"{"type":"row_finished","table":1,"line":2,"status":"error","checks":[{"method":"m","status":"error","actual":{"kind":"brand_new"}}]}"#;
        let found = events(&[line]);
        let failures = found.first().map_or(&[][..], Event::failures);
        assert_eq!(
            failures.first().map(Failure::note).as_deref(),
            Some("m got an answer temoc-ui does not know")
        );
    }

    #[test]
    fn a_row_can_fail_a_check_and_lose_the_connection() {
        let line = r#"{"type":"row_finished","table":1,"line":2,"status":"error","checks":[{"method":"m","status":"failed","expected":"1","actual":{"kind":"void"}}],"message":"Gone\nfor good"}"#;
        let found = events(&[line]);
        let notes: Vec<String> = found
            .first()
            .map_or(&[][..], Event::failures)
            .iter()
            .map(Failure::note)
            .collect();
        assert_eq!(notes, ["m expected 1, got no value", "Gone"]);
    }

    #[test]
    fn a_setup_failure_keeps_an_active_snooze() {
        let line = r#"{"type":"setup_failed","line":3,"phase":"make","status":"snoozed","actual":{"kind":"void"},"snoozed_until":"2099-12-31"}"#;
        let found = events(&[line]);
        let failure = found.first().map_or(&[][..], Event::failures).first();
        assert_eq!(
            failure.map(|failure| failure.result),
            Some(RunResult::Snoozed)
        );
        assert_eq!(
            failure.map(ToString::to_string).as_deref(),
            Some("line 3: make got no value (snoozed until 2099-12-31)")
        );
    }

    #[test]
    fn the_first_problem_with_a_run_is_the_one_shown() {
        let lines = [
            STARTED,
            r#"{"type":"run_error","kind":"server_died","message":"first"}"#,
            r#"{"type":"run_error","kind":"protocol_error","message":"second"}"#,
            r#"{"type":"run_finished","status":"error"}"#,
        ];
        assert_eq!(report(&lines, "").message.as_deref(), Some("first"));
    }

    #[test]
    fn a_cancelled_run_has_no_result() {
        let lines = [STARTED, r#"{"type":"run_finished","status":"cancelled"}"#];
        assert_eq!(
            report(&lines, ""),
            RunReport::stopped("temoc was stopped".into())
        );
    }

    #[test]
    fn a_status_temoc_ui_does_not_know_cannot_be_trusted() {
        let line = r#"{"type":"run_finished","status":"brand_new"}"#;
        assert_eq!(
            report(&[STARTED, line], "").message,
            Some(format!(
                "temoc printed something temoc-ui cannot read: {line}"
            ))
        );
    }

    #[test]
    fn nothing_after_a_problem_is_believed() {
        let lines = [STARTED, "not json", FINISHED];
        assert_eq!(report(&lines, "").result, RunResult::Error);
        let row = r#"{"type":"table_started","line":7}"#;
        assert_eq!(events(&["not json", row]), []);
    }

    #[test]
    fn blank_lines_are_not_a_problem() {
        assert_eq!(
            report(&[STARTED, "", "   ", FINISHED], "").result,
            RunResult::Passed
        );
    }

    #[test]
    fn a_program_that_succeeds_in_silence_is_not_an_older_temoc() {
        let transcript = Transcript::default();
        assert_eq!(
            transcript.report("", 0).message.as_deref(),
            Some("temoc stopped before finishing")
        );
        assert_eq!(
            transcript.report("", 2).message.as_deref(),
            Some("temoc-ui needs temoc 0.10 or later")
        );
    }

    #[test]
    fn only_the_first_line_of_stderr_is_shown() {
        assert_eq!(
            report(&[STARTED], "\nfirst\nsecond\n").message.as_deref(),
            Some("temoc stopped before finishing: first")
        );
    }
}
