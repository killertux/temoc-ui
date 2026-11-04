//! Understanding temoc: its test files and its output.

use crate::NotImplemented;
use std::path::Path;

/// What temoc would find in a markdown file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestFile {
    /// Whether the file has at least one `decisionTable` directive.
    pub is_test_file: bool,
    /// How many test tables can run.
    pub tables: usize,
}

/// What a temoc run reported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunReport {
    /// The overall result, such as `passed` or `failed`.
    pub result: String,
    /// The failures, in the order reported.
    pub failures: Vec<String>,
}

/// Looks for temoc tests in a markdown file.
///
/// # Errors
///
/// Fails until test detection is built.
pub fn inspect_file(_path: &Path) -> Result<TestFile, NotImplemented> {
    Err(NotImplemented)
}

/// Works out the result of a temoc run from what it printed.
///
/// # Errors
///
/// Fails until output parsing is built.
pub fn read_output(
    _stdout: &str,
    _stderr: &str,
    _exit_code: i32,
) -> Result<RunReport, NotImplemented> {
    Err(NotImplemented)
}
