//! Understanding temoc: its test files and its output.

use markdown::ParseOptions;
use markdown::mdast::{Definition, Node};
use std::fmt::{self, Display, Formatter};
use std::path::Path;
use std::{error, fs, io};

/// What temoc would find in a markdown file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TestFile {
    /// Whether the file has at least one `decisionTable` directive.
    pub is_test_file: bool,
    /// The name of each test table that can run, in document order.
    pub tables: Vec<String>,
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
fn find_tests(markdown: &str) -> Result<TestFile, InspectError> {
    let root = markdown::to_mdast(markdown, &ParseOptions::gfm())
        .map_err(|message| InspectError::Parse(message.to_string()))?;
    let mut found = TestFile::default();
    let mut heading = None;
    let mut nodes = root.children().into_iter().flatten().peekable();
    while let Some(node) = nodes.next() {
        match node {
            Node::Heading(_) => heading = Some(node.to_string()),
            Node::Definition(definition) => {
                let Some(class) = directive_class(definition) else {
                    continue;
                };
                found.is_test_file = true;
                if matches!(nodes.peek(), Some(Node::Table(_))) {
                    let name = heading.clone().unwrap_or_else(|| class.to_owned());
                    found.tables.push(name);
                }
            }
            _ => {}
        }
    }
    Ok(found)
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

/// The overall result of a temoc run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunResult {
    /// Everything that ran passed.
    Passed,
    /// At least one failure is real.
    Failed,
    /// Every failure is snoozed.
    Snoozed,
    /// temoc found nothing to run.
    NoTests,
    /// temoc stopped without a result.
    Error,
}

impl Display for RunResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Snoozed => "snoozed",
            Self::NoTests => "no tests",
            Self::Error => "error",
        })
    }
}

/// One failure reported by temoc.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    /// The line of the failure. temoc gives none for protocol problems.
    pub line: Option<usize>,
    /// What went wrong.
    pub message: String,
    /// The date a snoozed failure is snoozed until.
    pub snoozed_until: Option<String>,
}

impl Failure {
    /// Reads one line of temoc's output. A line in an unknown shape is kept as
    /// it is, so that no failure is ever lost.
    fn read(text: &str) -> Self {
        Self::read_known_shape(text).unwrap_or_else(|| Self {
            line: None,
            message: text.to_owned(),
            snoozed_until: None,
        })
    }

    /// Reads `Expected A, got B in FILE:LINE:COLUMN`, which may be followed by
    /// ` for method call NAME` and then ` -- snoozed until DATE`. Values can hold
    /// any text, so the fixed parts at the end are taken off first.
    fn read_known_shape(text: &str) -> Option<Self> {
        let (text, snoozed_until) = split_suffix(text, " -- snoozed until ");
        let (text, method) = split_suffix(text, " for method call ");
        let (outcome, place) = text.rsplit_once(" in ")?;
        let line = place.rsplit(':').nth(1)?.parse().ok()?;
        let (expected, got) = outcome.strip_prefix("Expected ")?.split_once(", got ")?;
        let (expected, got) = (show_value(expected), show_value(got));
        let message = match method {
            Some(method) => format!("{method} expected {expected}, got {got}"),
            None => format!("expected {expected}, got {got}"),
        };
        Some(Self {
            line: Some(line),
            message,
            snoozed_until: snoozed_until.map(String::from),
        })
    }
}

impl Display for Failure {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(line) = self.line {
            write!(f, "line {line}: ")?;
        }
        f.write_str(&self.message)?;
        if let Some(date) = &self.snoozed_until {
            write!(f, " (snoozed until {date})")?;
        }
        Ok(())
    }
}

fn split_suffix<'text>(text: &'text str, marker: &str) -> (&'text str, Option<&'text str>) {
    text.rsplit_once(marker)
        .map_or((text, None), |(text, suffix)| (text, Some(suffix)))
}

/// Puts a value from a failure line into words.
fn show_value(value: &str) -> String {
    if value == "NULL or VOID" {
        return "no value".into();
    }
    if let Some(message) = value.strip_prefix("Exception ") {
        return format!("exception {}", without_backticks(message));
    }
    without_backticks(value).into()
}

fn without_backticks(value: &str) -> &str {
    value
        .strip_prefix('`')
        .and_then(|value| value.strip_suffix('`'))
        .unwrap_or(value)
}

/// What a temoc run reported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunReport {
    /// The overall result.
    pub result: RunResult,
    /// The failures, in the order reported.
    pub failures: Vec<Failure>,
    /// temoc's message, when it stopped without a result.
    pub error: Option<String>,
}

/// Works out the result of a temoc run from what it printed.
#[must_use]
pub fn read_output(stdout: &str, stderr: &str, exit_code: i32) -> RunReport {
    let mut lines = stdout
        .lines()
        .skip_while(|line| !line.starts_with("Testing file "));
    let verdict = lines
        .next()
        .and_then(|line| line.rsplit_once("..."))
        .map(|(_, verdict)| verdict);
    let result = match (verdict, exit_code) {
        (Some("OK"), 0) => RunResult::Passed,
        (Some("SNOOZED"), 0) => RunResult::Snoozed,
        (Some("NONE"), 0) => RunResult::NoTests,
        // temoc prints SNOOZED if any failure is snoozed, even beside a real one.
        (Some("FAIL" | "SNOOZED"), _) => RunResult::Failed,
        _ => RunResult::Error,
    };
    RunReport {
        result,
        failures: lines
            .filter(|line| !line.trim().is_empty())
            .map(Failure::read)
            .collect(),
        error: (result == RunResult::Error)
            .then(|| error_message(stderr))
            .flatten(),
    }
}

/// temoc's own error comes last, after anything the Slim server printed.
fn error_message(stderr: &str) -> Option<String> {
    let message = stderr
        .lines()
        .rev()
        .find_map(|line| line.strip_prefix("Error: "))
        .unwrap_or_else(|| stderr.trim());
    (!message.is_empty()).then(|| message.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tables(markdown: &str) -> Vec<String> {
        find_tests(markdown).unwrap().tables
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
        assert_eq!(found.tables, Vec::<String>::new());
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
    fn a_file_that_cannot_be_read_is_an_error() {
        let result = inspect_file(Path::new("/no/such/file.md"));
        assert!(matches!(result, Err(InspectError::Read(_))));
    }

    #[test]
    fn a_failure_in_an_unknown_shape_is_kept() {
        let stdout = "Testing file a.md...FAIL\nDifferent ID in response. Expected 1 but got 2\n";
        let report = read_output(stdout, "", 1);
        assert_eq!(report.result, RunResult::Failed);
        assert_eq!(
            report.failures,
            [Failure {
                line: None,
                message: "Different ID in response. Expected 1 but got 2".into(),
                snoozed_until: None,
            }]
        );
    }

    #[test]
    fn a_value_can_contain_the_words_of_a_failure_line() {
        let stdout =
            "Testing file a.md...FAIL\nExpected `a in b`, got `c` in a.md:3:1 for method call sum";
        let report = read_output(stdout, "", 1);
        assert_eq!(
            report.failures.first().map(ToString::to_string).as_deref(),
            Some("line 3: sum expected a in b, got c")
        );
    }

    #[test]
    fn a_file_name_can_contain_dots() {
        let report = read_output("Testing file a...b.md...OK\n", "", 0);
        assert_eq!(report.result, RunResult::Passed);
    }

    #[test]
    fn a_passing_word_with_a_bad_exit_code_is_an_error() {
        let report = read_output("Testing file a.md...OK\n", "Error: closing failed", 1);
        assert_eq!(report.result, RunResult::Error);
        assert_eq!(report.error.as_deref(), Some("closing failed"));
    }

    #[test]
    fn no_output_at_all_is_an_error() {
        let report = read_output("", "", 127);
        assert_eq!(report.result, RunResult::Error);
        assert_eq!(report.error, None);
    }

    #[test]
    fn the_error_is_temocs_message_not_the_servers_noise() {
        let stderr = "server warning\nError: failed to fill whole buffer\n";
        let report = read_output("Testing file a.md...", stderr, 1);
        assert_eq!(report.error.as_deref(), Some("failed to fill whole buffer"));
    }

    #[test]
    fn an_error_without_the_usual_prefix_is_still_shown() {
        let report = read_output("", "sh: temoc: not found\n", 127);
        assert_eq!(report.error.as_deref(), Some("sh: temoc: not found"));
    }

    #[test]
    fn the_error_is_only_for_runs_with_no_result() {
        let stderr = "Error: Tests executed with error";
        let stdout = "Testing file a.md...FAIL\nExpected `5`, got `4` in a.md:2:1";
        assert_eq!(read_output(stdout, stderr, 1).error, None);
    }
}
