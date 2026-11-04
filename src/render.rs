//! Turning markdown into what appears on screen.

use crate::NotImplemented;

/// How a piece of markdown appears on screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preview {
    /// The text drawn.
    pub text: String,
    /// How the text is emphasized, such as `heading 1` or `plain`.
    pub style: String,
}

/// Renders a piece of markdown by itself. Returns `None` if nothing is drawn.
///
/// # Errors
///
/// Fails until rendering is built.
pub fn preview(_markdown: &str) -> Result<Option<Preview>, NotImplemented> {
    Err(NotImplemented)
}
