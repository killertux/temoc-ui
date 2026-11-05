//! Copying a code block. The block is below the fold of the `elements` sample,
//! so the page has to scroll to it first.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use std::path::Path;
use temoc_ui::app::Launch;
use temoc_ui::gui::Gui;
use temoc_ui::gui::egui::{Key, Vec2};

#[test]
fn copy_puts_the_whole_block_on_the_clipboard() -> Result<(), String> {
    let samples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../samples");
    let launch = Launch {
        current_dir: samples.join("elements"),
        ..Launch::default()
    };
    let window = Gui::start(&launch)?;
    let mut harness = Harness::builder()
        .with_size(Vec2::new(1000.0, 700.0))
        .build_ui_state(|ui, window: &mut Gui| window.show(ui), window);
    harness.run_ok();
    assert_eq!(harness.state().copied(), None);
    harness.key_press(Key::End);
    harness.run_ok();
    harness.get_by_label("Copy").click();
    harness.run_ok();
    assert_eq!(harness.state().copied(), Some("echo copied"));
    let says = |said: Option<String>| said.as_deref() == Some("Copied 11 characters");
    let mut status = harness.query_all_by(|node| says(node.label()) || says(node.value()));
    assert!(status.next().is_some(), "the status bar says so");
    Ok(())
}

#[test]
fn the_button_says_copied_for_a_moment() -> Result<(), String> {
    let samples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../samples");
    let launch = Launch {
        current_dir: samples.join("elements"),
        ..Launch::default()
    };
    let window = Gui::start(&launch)?;
    let mut harness = Harness::builder()
        .with_size(Vec2::new(1000.0, 700.0))
        .build_ui_state(|ui, window: &mut Gui| window.show(ui), window);
    harness.run_ok();
    harness.key_press(Key::End);
    harness.run_ok();
    harness.get_by_label("Copy").click();
    harness.run_ok();
    assert!(harness.query_by_label("✓ Copied").is_some());
    assert!(harness.query_by_label("Copy").is_none());
    // Each step is a quarter of a second.
    for _ in 0..12 {
        harness.step();
    }
    assert!(harness.query_by_label("Copy").is_some());
    assert!(harness.query_by_label("✓ Copied").is_none());
    Ok(())
}
