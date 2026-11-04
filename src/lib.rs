//! temoc-ui: a terminal viewer and test runner for temoc markdown files.
//!
//! The behavior of this crate is described by the specification in `spec/`.
//! Most of it does not exist yet, so these items are the interface that the
//! specification's fixtures drive.

pub mod app;
pub mod page;
pub mod render;
pub mod search;
pub mod temoc;
pub mod workspace;

/// A feature described by the specification that has not been built yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotImplemented;
