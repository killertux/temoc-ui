//! What a build without the `gui` feature says. See `spec/gui/opening.md`.
//!
//! The spec cannot show this, because its fixtures are built with the feature.

#![cfg(not(feature = "gui"))]

use std::process::Command;

#[test]
fn a_build_without_the_gui_says_so_and_exits() {
    let output = Command::new(env!("CARGO_BIN_EXE_temoc-ui"))
        .arg("--gui")
        .output()
        .expect("temoc-ui starts");

    assert_eq!(
        String::from_utf8_lossy(&output.stderr).trim(),
        "This build has no GUI. Rebuild it with the gui feature."
    );
    assert!(!output.status.success());
}
