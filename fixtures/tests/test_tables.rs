//! Where the parts of a test table end up. The spec does not test positions,
//! as `spec/gui/writing-specs.md` says, but these once went wrong.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};
use temoc_ui::app::Launch;
use temoc_ui::gui::egui::{Key, Modifiers, Rect, Vec2};
use temoc_ui::gui::{Gui, tags};

const HEIGHT: f32 = 700.0;

/// A window that ran a file and then moved down this many tables.
fn ran(workspace: &str, file: &str, tables: usize) -> Result<Harness<'static, Gui>, String> {
    // The samples start `calculator-server`.
    let server = Path::new(env!("CARGO_BIN_EXE_calculator-server"));
    let samples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../samples");
    let launch = Launch {
        current_dir: samples.join(workspace),
        arguments: vec![file.to_owned()],
        extra_path: server.parent().map(Path::to_path_buf).into_iter().collect(),
        ..Launch::default()
    };
    let window = Gui::start(&launch)?;
    let mut harness = Harness::builder()
        .with_size(Vec2::new(1000.0, HEIGHT))
        .build_ui_state(|ui, window: &mut Gui| window.show(ui), window);
    press(&mut harness, Modifiers::COMMAND, Key::R);
    let deadline = Instant::now() + Duration::from_mins(1);
    while harness.state().is_busy() {
        if Instant::now() > deadline {
            return Err("the run did not finish".into());
        }
        harness.step();
        sleep(Duration::from_millis(10));
    }
    harness.run_ok();
    for _ in 0..tables {
        press(&mut harness, Modifiers::ALT, Key::ArrowDown);
    }
    Ok(harness)
}

fn press(harness: &mut Harness<'static, Gui>, modifiers: Modifiers, key: Key) {
    harness.key_press_modifiers(modifiers, key);
    harness.run_ok();
}

/// Where the last thing with this tag is.
fn last_tagged(harness: &Harness<'static, Gui>, tag: &'static str) -> Result<Rect, String> {
    let found = harness.query_all_by(move |node| node.description().as_deref() == Some(tag));
    let last = found.last().ok_or(format!("no {tag} on the page"))?;
    Ok(last.rect())
}

#[test]
fn a_verdict_sits_below_the_notes_of_the_last_row() -> Result<(), String> {
    let window = ran("calculator", "failing.md", 0)?;
    let note = last_tagged(&window, tags::NOTE)?;
    let verdict = last_tagged(&window, tags::VERDICT)?;
    assert!(note.bottom() <= verdict.top(), "{note:?} and {verdict:?}");
    Ok(())
}

#[test]
fn focusing_a_table_brings_it_into_view() -> Result<(), String> {
    let before = ran("outcomes", "errors.md", 0)?;
    let before = last_tagged(&before, tags::VERDICT)?;
    assert!(before.bottom() > HEIGHT, "it starts out of view");
    let after = ran("outcomes", "errors.md", 2)?;
    let after = last_tagged(&after, tags::VERDICT)?;
    assert!(after.bottom() <= HEIGHT);
    Ok(())
}
