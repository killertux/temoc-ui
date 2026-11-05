//! What the spec leaves to the eye in the graph: moving the view, and the keys
//! that move the selection.

use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use std::path::Path;
use temoc_ui::app::Launch;
use temoc_ui::gui::egui::{
    Event, Key, Modifiers, MouseWheelSource, MouseWheelUnit, PointerButton, Pos2, TouchPhase, Vec2,
};
use temoc_ui::gui::{Gui, tags};

fn graph_of(workspace: &str) -> Result<Harness<'static, Gui>, String> {
    let samples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../samples");
    let launch = Launch {
        current_dir: samples.join(workspace),
        ..Launch::default()
    };
    let window = Gui::start(&launch)?;
    let mut harness = Harness::builder()
        .with_size(Vec2::new(1000.0, 700.0))
        .build_ui_state(|ui, window: &mut Gui| window.show(ui), window);
    harness.run_ok();
    press(&mut harness, Modifiers::COMMAND, Key::G);
    while harness.state().graph_is_moving() {
        harness.step();
    }
    harness.run_ok();
    Ok(harness)
}

fn press(harness: &mut Harness<'static, Gui>, modifiers: Modifiers, key: Key) {
    harness.key_press_modifiers(modifiers, key);
    harness.run_ok();
}

/// Each file of the graph and where it is.
fn places(harness: &Harness<'static, Gui>) -> Vec<(String, Pos2)> {
    let tag = Some(tags::GRAPH_NODE);
    let nodes = harness.query_all_by(move |node| node.description().as_deref() == tag);
    nodes
        .map(|node| {
            let name = node.accesskit_node().label().unwrap_or_default();
            (name, node.rect().center())
        })
        .collect()
}

fn selected(harness: &Harness<'static, Gui>) -> Option<String> {
    let tag = Some(tags::GRAPH_NODE);
    let mut nodes = harness.query_all_by(move |node| node.description().as_deref() == tag);
    let node = nodes.find(|node| node.accesskit_node().is_selected() == Some(true));
    node.and_then(|node| node.accesskit_node().label())
}

fn drag_background(harness: &mut Harness<'static, Gui>, from: Pos2, by: Vec2) {
    let button = |pos, pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    harness.hover_at(from);
    harness.run_ok();
    harness.event(button(from, true));
    harness.run_ok();
    harness.hover_at(from + by);
    harness.run_ok();
    harness.event(button(from + by, false));
    harness.run_ok();
}

#[test]
fn dragging_the_background_moves_every_file_and_fit_brings_them_back() -> Result<(), String> {
    let mut window = graph_of("docs")?;
    let before = places(&window);
    let by = Vec2::new(30.0, 20.0);
    drag_background(&mut window, Pos2::new(560.0, 640.0), by);
    let dragged = places(&window);
    for ((_, was), (_, now)) in before.iter().zip(&dragged) {
        assert!((*now - *was - by).length() < 12.0, "{was:?} to {now:?}");
    }
    window.get_by_label("Fit").click();
    window.run_ok();
    assert_eq!(places(&window), before);
    Ok(())
}

#[test]
fn the_wheel_zooms_in_on_the_pointer() -> Result<(), String> {
    let mut window = graph_of("docs")?;
    let spread = |places: &[(String, Pos2)]| {
        let first = places.first().map_or(Pos2::ZERO, |(_, place)| *place);
        let last = places.last().map_or(Pos2::ZERO, |(_, place)| *place);
        (first - last).length()
    };
    let before = spread(&places(&window));
    window.hover_at(Pos2::new(560.0, 350.0));
    window.run_ok();
    window.event(Event::MouseWheel {
        unit: MouseWheelUnit::Point,
        delta: Vec2::new(0.0, 80.0),
        phase: TouchPhase::Move,
        source: MouseWheelSource::Wheel,
        modifiers: Modifiers::NONE,
    });
    window.run_ok();
    assert!(spread(&places(&window)) > before);
    Ok(())
}

#[test]
fn an_arrow_key_selects_the_nearest_file_that_way() -> Result<(), String> {
    let mut window = graph_of("docs")?;
    let all = places(&window);
    let start = selected(&window).ok_or("nothing selected")?;
    let from = all.iter().find(|(name, _)| *name == start);
    let from = from.ok_or("the selection is not in the graph")?.1;
    let is_right_of = |place: &Pos2| {
        let apart = *place - from;
        apart.x > 0.0 && apart.x >= apart.y.abs()
    };
    let right = all.iter().filter(|(_, place)| is_right_of(place));
    let nearest = right.min_by(|one, other| {
        let distance = |place: Pos2| (place - from).length();
        distance(one.1).total_cmp(&distance(other.1))
    });
    press(&mut window, Modifiers::NONE, Key::ArrowRight);
    let expected = nearest.map_or(start, |(name, _)| name.clone());
    assert_eq!(selected(&window), Some(expected));
    Ok(())
}

#[test]
fn going_back_leaves_the_graph() -> Result<(), String> {
    let mut window = graph_of("docs")?;
    press(&mut window, Modifiers::ALT, Key::ArrowLeft);
    assert_eq!(places(&window), Vec::new());
    Ok(())
}
