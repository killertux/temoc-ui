//! temoc-ui: a terminal viewer and test runner for temoc markdown files.
//!
//! The behavior of this crate is described by the specification in `spec/`.
//! Most of it does not exist yet, so these items are the interface that the
//! specification's fixtures drive.

pub mod app;
pub mod clipboard;
pub mod document;
pub mod focus;
pub mod graph;
#[cfg(feature = "gui")]
pub mod gui;
pub mod help;
pub mod history;
pub mod links;
pub mod page;
pub mod render;
pub mod run;
pub mod search;
pub mod session;
pub mod temoc;
pub mod terminal;
pub mod theme;
pub mod tree;
pub mod workspace;

/// A feature described by the specification that has not been built yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotImplemented;
