//! Where the search field sits, and how a link to nothing looks. The spec does
//! not test positions, as `spec/gui/writing-specs.md` says.

use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use std::path::Path;
use temoc_ui::app::Launch;
use temoc_ui::gui::egui::accesskit::Role;
use temoc_ui::gui::egui::{Event, Key, Vec2};
use temoc_ui::gui::{Gui, tags};

const WIDTH: f32 = 1200.0;

fn docs() -> Result<Harness<'static, Gui>, String> {
    let samples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../samples");
    let launch = Launch {
        current_dir: samples.join("docs"),
        ..Launch::default()
    };
    let window = Gui::start(&launch)?;
    let mut harness = Harness::builder()
        .with_size(Vec2::new(WIDTH, 700.0))
        .build_ui_state(|ui, window: &mut Gui| window.show(ui), window);
    harness.run_ok();
    Ok(harness)
}

#[test]
fn the_search_field_sits_in_the_middle_of_the_window() -> Result<(), String> {
    let window = docs()?;
    let field = window.get_by_label("Search files and headings").rect();
    assert!((field.center().x - WIDTH / 2.0).abs() < 1.0, "{field:?}");
    Ok(())
}

#[test]
fn searching_types_in_the_field_and_shows_results_under_it() -> Result<(), String> {
    let mut window = docs()?;
    let field = window.get_by_label("Search files and headings").rect();
    window.key_press(Key::Slash);
    window.event(Event::Text("/".into()));
    window.run_ok();
    let tag = Some(tags::SEARCH);
    let typing = window
        .query_all_by(move |node| node.description().as_deref() == tag)
        .next()
        .ok_or("the field has no search in it")?
        .rect();
    assert!(field.contains_rect(typing), "{typing:?} in {field:?}");
    let tag = Some(tags::SEARCH_RESULT);
    let result = window
        .query_all_by(move |node| node.description().as_deref() == tag)
        .next()
        .ok_or("no results")?
        .rect();
    assert!(result.top() > field.bottom(), "{result:?} under {field:?}");
    Ok(())
}

#[test]
fn only_a_link_to_a_missing_file_is_marked_as_one() -> Result<(), String> {
    let window = docs()?;
    let tag = Some(tags::MISSING_LINK);
    let missing = window.query_all_by(move |node| node.description().as_deref() == tag);
    let names: Vec<String> = missing
        .filter(|node| node.accesskit_node().role() == Role::Link)
        .filter_map(|node| node.accesskit_node().label())
        .collect();
    assert_eq!(names, ["Missing page"]);
    Ok(())
}

#[test]
fn a_click_in_the_field_keeps_the_search_open() -> Result<(), String> {
    let mut window = docs()?;
    window.get_by_label("Search files and headings").click();
    window.run_ok();
    let tag = Some(tags::SEARCH);
    let field = window
        .query_all_by(move |node| node.description().as_deref() == tag)
        .next()
        .ok_or("no search")?;
    field.click();
    window.run_ok();
    let tag = Some(tags::SEARCH);
    let mut open = window.query_all_by(move |node| node.description().as_deref() == tag);
    assert!(open.next().is_some(), "the search stays open");
    Ok(())
}
