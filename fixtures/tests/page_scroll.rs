//! How far a page stays scrolled. See the history in `spec/gui/navigation.md`.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use std::path::Path;
use temoc_ui::app::Launch;
use temoc_ui::gui::egui::accesskit::Role;
use temoc_ui::gui::egui::{Key, Modifiers, Vec2};
use temoc_ui::gui::{Gui, Position};

/// The window at the long page, scrolled to its end and left behind.
fn left_the_end_of_a_long_page() -> Result<Harness<'static, Gui>, String> {
    let samples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../samples");
    let launch = Launch {
        current_dir: samples.join("docs"),
        ..Launch::default()
    };
    let window = Gui::start(&launch)?;
    let mut harness = Harness::builder()
        .with_size(Vec2::new(1000.0, 700.0))
        .build_ui_state(|ui, window: &mut Gui| window.show(ui), window);
    harness.run_ok();
    click_long_page(&mut harness);
    press(&mut harness, Modifiers::NONE, Key::End);
    press(&mut harness, Modifiers::ALT, Key::ArrowLeft);
    Ok(harness)
}

fn click_long_page(harness: &mut Harness<'static, Gui>) {
    harness
        .get_by_role_and_label(Role::Link, "Long page")
        .click();
    harness.run_ok();
}

fn press(harness: &mut Harness<'static, Gui>, modifiers: Modifiers, key: Key) {
    harness.key_press_modifiers(modifiers, key);
    harness.run_ok();
}

#[test]
fn going_forward_finds_a_page_as_it_was_left() -> Result<(), String> {
    let mut window = left_the_end_of_a_long_page()?;
    press(&mut window, Modifiers::ALT, Key::ArrowRight);
    assert_eq!(window.state().page_position(), Some(Position::End));
    Ok(())
}

#[test]
fn a_new_visit_to_a_page_starts_at_the_top() -> Result<(), String> {
    let mut window = left_the_end_of_a_long_page()?;
    click_long_page(&mut window);
    assert_eq!(window.state().page_position(), Some(Position::Start));
    Ok(())
}
