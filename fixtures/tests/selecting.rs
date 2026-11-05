//! Selecting text across the runs of a paragraph, and in a test table. Samples
//! cannot change for this, so the paragraph is written here.

use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use std::fs;
use std::path::Path;
use temoc_ui::app::Launch;
use temoc_ui::gui::egui::{Event, Modifiers, PointerButton, Pos2, Rect, Vec2};
use temoc_ui::gui::{Gui, tags};

const PARAGRAPH: &str = "Press (`Ctrl-c`) or **bo**ld now";

fn window(launch: &Launch) -> Result<Harness<'static, Gui>, String> {
    let window = Gui::start(launch)?;
    let mut harness = Harness::builder()
        .with_size(Vec2::new(1000.0, 700.0))
        .build_ui_state(|ui, window: &mut Gui| window.show(ui), window);
    harness.run_ok();
    Ok(harness)
}

fn button(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}

fn drag(harness: &mut Harness<'static, Gui>, from: Pos2, to: Pos2) {
    harness.hover_at(from);
    harness.run_ok();
    harness.event(button(from, true));
    harness.run_ok();
    harness.hover_at(to);
    harness.run_ok();
    harness.event(button(to, false));
    harness.run_ok();
}

/// The whole of a line of text, from before its first character to after its
/// last.
fn across(rect: Rect) -> (Pos2, Pos2) {
    let y = rect.center().y;
    (
        Pos2::new(rect.left() + 1.0, y),
        Pos2::new(rect.right() - 1.0, y),
    )
}

#[test]
fn a_paragraph_is_copied_the_way_it_shows() -> Result<(), String> {
    let folder = std::env::temp_dir().join(format!("temoc-ui-selecting-{}", std::process::id()));
    fs::create_dir_all(&folder).map_err(|error| error.to_string())?;
    let readme = folder.join("README.md");
    fs::write(&readme, format!("{PARAGRAPH}\n")).map_err(|error| error.to_string())?;
    let launch = Launch {
        current_dir: folder.clone(),
        ..Launch::default()
    };
    let mut window = window(&launch)?;
    let text = "Press (Ctrl-c) or bold now";
    let rect = window.get_by_label(text).rect();
    let (from, to) = across(rect);
    drag(&mut window, from, to);
    window.event(Event::Copy);
    window.run_ok();
    assert_eq!(window.state().copied(), Some(text));
    fs::remove_dir_all(&folder).map_err(|error| error.to_string())
}

#[test]
fn text_in_a_test_table_can_be_selected() -> Result<(), String> {
    let samples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../samples");
    let launch = Launch {
        current_dir: samples.join("calculator"),
        arguments: vec!["passing.md".into()],
        ..Launch::default()
    };
    let mut harness = window(&launch)?;
    let (from, to) = across(harness.get_by_label("sum?").rect());
    drag(&mut harness, from, to);
    assert_eq!(harness.state().selected(), Some("sum?"));
    Ok(())
}

#[test]
fn a_click_on_the_text_of_a_test_row_focuses_it() -> Result<(), String> {
    let samples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../samples");
    let launch = Launch {
        current_dir: samples.join("calculator"),
        arguments: vec!["passing.md".into()],
        ..Launch::default()
    };
    let mut harness = window(&launch)?;
    let spot = harness.get_by_label("mul?").rect().center();
    harness.hover_at(spot);
    harness.run_ok();
    // A press held for long is no click, so this only takes the one frame.
    harness.event(button(spot, true));
    harness.step();
    harness.event(button(spot, false));
    harness.run_ok();
    let tag = Some(tags::TEST_TABLE);
    let mut headers = harness.query_all_by(move |node| node.description().as_deref() == tag);
    let focused = headers.find(|header| header.accesskit_node().is_selected() == Some(true));
    let name = focused.and_then(|header| header.accesskit_node().label());
    assert_eq!(name.as_deref(), Some("Multiplication"));
    Ok(())
}
