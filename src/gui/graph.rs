//! The graph view: the files of the workspace and how they link. See
//! `spec/gui/graph.md`, and `spec/viewer/graph.md` for what it shows.

mod layout;

use super::look::{Palette, STRONG};
use super::{mark_of, result_colour, tagged, tags};
use crate::links::LinkGraph;
use eframe::egui::accesskit::Role;
use eframe::egui::{
    self, Align2, Button, Color32, FontFamily, FontId, Id, Pos2, Rect, Response, RichText, Sense,
    Shape, Stroke, Ui, UiBuilder, Vec2, WidgetInfo, vec2,
};
pub(super) use layout::Heading;
use layout::{Layout, nearest};
use std::collections::HashMap;
use std::path::Path;

/// How many steps of the layout go by in a frame.
const STEPS_PER_FRAME: usize = 6;
/// The radius of a file with no links.
const SMALLEST: f32 = 8.0;
/// The radius of a file with many links.
const LARGEST: f32 = 22.0;
/// The smallest radius with room for the mark of a result.
const WITH_MARK: f32 = 12.0;
/// How much each link adds to the radius of a file.
const GROWTH: f32 = 0.5;
/// How much of the colour of a file is left when it fades.
const FADED: f32 = 0.35;
/// How much room the edges of the view keep around the files for their names.
const FIT_MARGIN: f32 = 70.0;
/// The most `Fit` zooms in.
const FIT_ZOOM: f32 = 1.2;
const ZOOM_RANGE: (f32, f32) = (0.2, 4.0);
/// How much one press of a zoom button zooms.
const ZOOM_STEP: f32 = 1.25;
/// How long and how wide an arrowhead is.
const ARROWHEAD: Vec2 = vec2(9.0, 4.5);

/// What the graph shows of a file besides its links.
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct Look {
    pub is_test: bool,
    /// The name of its result, if it has one.
    pub result: Option<&'static str>,
}

/// How the graph is looked at.
#[derive(Debug, Clone, Copy)]
struct Camera {
    zoom: f32,
    /// How far the middle of the graph is from the middle of the view.
    pan: Vec2,
    /// Whether everything is kept in view.
    fits: bool,
}

impl Camera {
    fn on_screen(self, area: Rect, place: Pos2) -> Pos2 {
        area.center() + self.pan + place.to_vec2() * self.zoom
    }

    /// Looks at all of `bounds`.
    fn fitted(bounds: Rect, area: Rect) -> Self {
        let room = (area.size() - Vec2::splat(2.0 * FIT_MARGIN)).max(Vec2::splat(1.0));
        let size = bounds.size().max(Vec2::splat(1.0));
        let zoom = (room.x / size.x).min(room.y / size.y).min(FIT_ZOOM);
        Self {
            zoom,
            pan: -bounds.center().to_vec2() * zoom,
            fits: true,
        }
    }

    /// Zooms by `factor`, keeping what is under `spot` where it is.
    fn zoomed(self, factor: f32, spot: Pos2, area: Rect) -> Self {
        let zoom = (self.zoom * factor).clamp(ZOOM_RANGE.0, ZOOM_RANGE.1);
        let from_middle = spot - area.center();
        let kept = zoom / self.zoom;
        Self {
            zoom,
            pan: from_middle + (self.pan - from_middle) * kept,
            fits: false,
        }
    }
}

/// What was drawn last, for anyone who cannot see it.
#[derive(Debug, Clone, Default)]
struct Drawn {
    lit: Vec<usize>,
    arrowheads: usize,
}

/// What clicking on the view asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Asked {
    /// Open this file.
    Open(String),
}

/// The graph of a workspace, while it shows.
#[derive(Debug)]
pub(super) struct GraphView {
    links: LinkGraph,
    layout: Layout,
    selected: Option<usize>,
    hovered: Option<usize>,
    /// Where the files that were dragged went.
    moved: HashMap<usize, Pos2>,
    camera: Camera,
    /// What the graph shows of each file besides its links.
    looks: Vec<Look>,
    drawn: Drawn,
}

impl GraphView {
    /// Reads the links of a workspace, with `document` selected.
    pub fn open(workspace: &Path, document: Option<&str>) -> Self {
        let links = LinkGraph::read(workspace);
        let chosen = document.and_then(|document| links.position(document));
        let selected = chosen.or(links.start());
        let layout = Layout::new(links.files().len(), links.between().iter().copied());
        Self {
            links,
            layout,
            selected,
            hovered: None,
            moved: HashMap::new(),
            camera: Camera {
                zoom: 1.0,
                pan: Vec2::ZERO,
                fits: true,
            },
            looks: Vec::new(),
            drawn: Drawn::default(),
        }
    }

    pub fn files(&self) -> &[String] {
        self.links.files()
    }

    pub fn is_moving(&self) -> bool {
        !self.layout.is_settled()
    }

    pub fn selected_file(&self) -> Option<&String> {
        self.links.name(self.selected?)
    }

    /// The files the graph is drawing attention to.
    pub fn lit_files(&self) -> Vec<String> {
        let lit = self.drawn.lit.iter();
        lit.filter_map(|file| self.links.name(*file).cloned())
            .collect()
    }

    pub fn arrowheads(&self) -> usize {
        self.drawn.arrowheads
    }

    pub fn select(&mut self, name: &str) {
        if let Some(file) = self.links.position(name) {
            self.selected = Some(file);
        }
    }

    pub fn select_start(&mut self) {
        self.selected = self.links.start().or(self.selected);
    }

    /// Selects the nearest file this way, if there is one.
    pub fn select_towards(&mut self, heading: Heading) {
        let places = self.places();
        let next = self
            .selected
            .and_then(|file| nearest(&places, file, heading));
        self.selected = next.or(self.selected);
    }

    /// Where each file is, in the order of the files.
    fn places(&self) -> Vec<Pos2> {
        (0..self.links.files().len())
            .map(|file| {
                let moved = self.moved.get(&file).copied();
                moved.or(self.layout.place(file)).unwrap_or(Pos2::ZERO)
            })
            .collect()
    }

    /// How big a file is, which goes by how many links it has.
    fn radius(&self, file: usize) -> f32 {
        let between = self.links.between().iter();
        let links = between.filter(|(from, to)| *from == file || *to == file);
        let links = links.fold(0.0_f32, |count, _| count + 1.0);
        let size = (SMALLEST + GROWTH * links).min(LARGEST);
        let has_result = self
            .looks
            .get(file)
            .is_some_and(|look| look.result.is_some());
        let size = if has_result {
            size.max(WITH_MARK)
        } else {
            size
        };
        size * self.camera.zoom.clamp(0.6, 1.0)
    }

    /// Draws the graph, and says what was asked of it.
    pub fn show(&mut self, ui: &mut Ui, looks: Vec<Look>, palette: &Palette) -> Option<Asked> {
        self.looks = looks;
        for _ in 0..STEPS_PER_FRAME {
            self.layout.step();
        }
        if self.is_moving() {
            ui.ctx().request_repaint();
        }
        let area = ui.available_rect_before_wrap();
        let background = ui.allocate_rect(area, Sense::click_and_drag());
        tagged(&background, tags::GRAPH, "Graph");
        self.pan_and_zoom(ui, &background, area);
        if self.camera.fits {
            self.camera = Camera::fitted(self.layout.bounds(), area);
        }
        let asked = self.nodes(ui, area);
        self.paint(ui, area, palette);
        self.controls(ui, area);
        asked
    }

    fn pan_and_zoom(&mut self, ui: &Ui, background: &Response, area: Rect) {
        if background.dragged() {
            self.camera.pan += background.drag_delta();
            self.camera.fits = false;
        }
        let scrolled = ui.input(|input| input.smooth_scroll_delta.y);
        let spot = ui.input(|input| input.pointer.hover_pos());
        if let Some(spot) = spot.filter(|spot| area.contains(*spot))
            && scrolled != 0.0
        {
            self.camera = self.camera.zoomed((scrolled / 200.0).exp(), spot, area);
        }
    }

    /// Lets the pointer at the files.
    fn nodes(&mut self, ui: &Ui, area: Rect) -> Option<Asked> {
        let mut asked = None;
        let mut hovered = None;
        for (file, place) in self.places().into_iter().enumerate() {
            let center = self.camera.on_screen(area, place);
            let size = Vec2::splat(2.0 * self.radius(file));
            let rect = Rect::from_center_size(center, size);
            let id = node_id(ui, file);
            let response = ui.interact(rect, id, Sense::click_and_drag());
            self.describe(&response, file);
            if response.hovered() || response.dragged() {
                hovered = Some(file);
            }
            if response.dragged() {
                let moved = place + response.drag_delta() / self.camera.zoom;
                self.moved.insert(file, moved);
            }
            if response.clicked() {
                if self.selected == Some(file) {
                    asked = self.links.name(file).cloned().map(Asked::Open);
                }
                self.selected = Some(file);
            }
        }
        self.hovered = hovered;
        asked
    }

    fn describe(&self, response: &Response, file: usize) {
        let name = self.links.name(file).map_or("", String::as_str);
        let selected = self.selected == Some(file);
        response.widget_info(|| WidgetInfo::selected(Role::Button, true, selected, name));
        response.ctx.accesskit_node_builder(response.id, |node| {
            node.set_description(tags::GRAPH_NODE);
            node.set_selected(selected);
        });
    }

    fn paint(&mut self, ui: &Ui, area: Rect, palette: &Palette) {
        let attention = self.hovered.or(self.selected);
        let between = self.links.between();
        let touches = |file: usize| {
            let close = |(from, to): &&(usize, usize)| {
                (Some(*from) == attention && *to == file)
                    || (Some(*to) == attention && *from == file)
            };
            Some(file) == attention || between.iter().find(close).is_some()
        };
        let places = self.places();
        let lit: Vec<usize> = (0..places.len()).filter(|file| touches(*file)).collect();
        let screen: Vec<Pos2> = places
            .iter()
            .map(|place| self.camera.on_screen(area, *place))
            .collect();
        let painter = ui.painter_at(area);
        let arrowheads = self.lines(&painter, &screen, attention, palette);
        let spots: Vec<Spot> = screen
            .iter()
            .enumerate()
            .map(|(file, center)| Spot {
                center: *center,
                radius: self.radius(file),
                lit: lit.contains(&file),
                selected: self.selected == Some(file),
            })
            .collect();
        for (file, spot) in spots.iter().enumerate() {
            let look = self.looks.get(file).copied().unwrap_or_default();
            paint_file(&painter, *spot, look, palette);
            if let Some(result) = look.result {
                ui.ctx().accesskit_node_builder(node_id(ui, file), |node| {
                    node.set_value(result);
                });
            }
        }
        // After every file, so that no file covers the name of another.
        for (name, spot) in self.links.files().iter().zip(&spots) {
            paint_name(&painter, *spot, name, palette);
        }
        self.drawn = Drawn { lit, arrowheads };
    }

    /// Draws a line for each pair of linked files, and says how many arrowheads
    /// went on them.
    fn lines(
        &self,
        painter: &egui::Painter,
        screen: &[Pos2],
        attention: Option<usize>,
        palette: &Palette,
    ) -> usize {
        let between = self.links.between();
        let mut arrowheads = 0;
        for (from, to) in between {
            let both_ways = between.contains(&(*to, *from));
            let (Some(start), Some(end)) = (screen.get(*from), screen.get(*to)) else {
                continue;
            };
            let lit = attention == Some(*from) || attention == Some(*to);
            if !both_ways || from < to {
                let colour = if lit { palette.bar } else { palette.edge };
                let width = if lit { 1.5 } else { 1.0 };
                painter.line_segment([*start, *end], Stroke::new(width, colour));
            }
            if lit {
                arrowhead(painter, *start, *end, self.radius(*to), palette.bar);
                arrowheads += 1;
            }
        }
        arrowheads
    }

    /// The buttons over the corner of the view.
    fn controls(&mut self, ui: &mut Ui, area: Rect) {
        let spot = Rect::from_min_size(area.min + vec2(12.0, 12.0), vec2(160.0, 28.0));
        let mut bar = ui.new_child(UiBuilder::new().id_salt("graph controls").max_rect(spot));
        bar.horizontal(|ui| {
            if ui.add(Button::new("-")).on_hover_text("Zoom out").clicked() {
                self.camera = self.camera.zoomed(1.0 / ZOOM_STEP, area.center(), area);
                ui.ctx().request_repaint();
            }
            if ui.add(Button::new("+")).on_hover_text("Zoom in").clicked() {
                self.camera = self.camera.zoomed(ZOOM_STEP, area.center(), area);
                ui.ctx().request_repaint();
            }
            if ui.add(Button::new("Fit")).clicked() {
                self.moved.clear();
                self.camera.fits = true;
                ui.ctx().request_repaint();
            }
        });
    }

    /// Lists the links of the selected file beside the graph, and says what was
    /// asked of it.
    pub fn details(&mut self, ui: &mut Ui, palette: &Palette) -> Option<Asked> {
        let file = self.selected?;
        let name = self.links.name(file)?.clone();
        let font = FontId::new(14.0, FontFamily::Name(STRONG.into()));
        ui.label(RichText::new(&name).font(font).color(palette.heading));
        let distance = self.distance_text(file);
        let response = ui.label(RichText::new(&distance).color(palette.quote));
        tagged(&response, tags::GRAPH_DISTANCE, &distance);
        ui.add_space(10.0);
        let open = Button::new(RichText::new("Open file").color(palette.bar_text));
        let mut asked = None;
        if ui.add(open.fill(palette.bar)).clicked() {
            asked = Some(Asked::Open(name));
        }
        let links_out = self.links.links_out(file);
        let links_in = self.links.links_in(file);
        let chosen = [
            self.link_list(ui, "Links out", &links_out, tags::GRAPH_LINK_OUT, palette),
            self.link_list(ui, "Links in", &links_in, tags::GRAPH_LINK_IN, palette),
        ];
        if let Some(chosen) = chosen.into_iter().flatten().next() {
            self.select(&chosen);
        }
        asked
    }

    fn distance_text(&self, file: usize) -> String {
        if self.links.start() == Some(file) {
            return "The start page".into();
        }
        match self.links.distance(file) {
            None => "Not reachable from the start page".into(),
            Some(1) => "1 link from the start page".into(),
            Some(links) => format!("{links} links from the start page"),
        }
    }

    /// Lists some links, and says which one was clicked.
    fn link_list(
        &self,
        ui: &mut Ui,
        title: &str,
        links: &[String],
        tag: &str,
        palette: &Palette,
    ) -> Option<String> {
        ui.add_space(16.0);
        ui.label(RichText::new(title).size(12.0).color(palette.quote));
        if links.is_empty() {
            ui.label(RichText::new("None").color(palette.quote));
        }
        let mut clicked = None;
        for link in links {
            let text = RichText::new(link).size(12.5);
            let response = if self.links.position(link).is_some() {
                ui.add(egui::Link::new(text.color(palette.link)))
            } else {
                ui.label(text.color(palette.quote))
            };
            tagged(&response, tag, link);
            if response.clicked() {
                clicked = Some(link.clone());
            }
        }
        clicked
    }
}

/// What tells a file of the graph apart.
fn node_id(ui: &Ui, file: usize) -> Id {
    ui.scope_id().with(("graph file", file))
}

/// Where a file is drawn, and how much it stands out.
#[derive(Debug, Clone, Copy)]
struct Spot {
    center: Pos2,
    radius: f32,
    lit: bool,
    selected: bool,
}

fn paint_file(painter: &egui::Painter, spot: Spot, look: Look, palette: &Palette) {
    let fade = |colour: Color32| {
        if spot.lit {
            colour
        } else {
            colour.gamma_multiply(FADED)
        }
    };
    let Spot { center, radius, .. } = spot;
    match look.result {
        Some(result) => {
            painter.circle_filled(center, radius, fade(result_colour(result, palette)));
            if let Some(mark) = mark_of(result) {
                let font = FontId::monospace(11.0);
                painter.text(
                    center,
                    Align2::CENTER_CENTER,
                    mark,
                    font,
                    fade(palette.page),
                );
            }
        }
        // A file with tests is a ring until it has a result.
        None if look.is_test => {
            painter.circle_filled(center, radius, palette.page);
            painter.circle_stroke(center, radius - 1.0, Stroke::new(2.0, fade(palette.line)));
        }
        None => {
            painter.circle_filled(center, radius, fade(palette.line));
        }
    }
    if spot.selected {
        painter.circle_stroke(center, radius + 3.0, Stroke::new(2.0, palette.bar));
    }
}

/// The name of a file, below it.
fn paint_name(painter: &egui::Painter, spot: Spot, name: &str, palette: &Palette) {
    let shown = name.strip_suffix(".md").unwrap_or(name);
    let colour = if spot.lit {
        palette.heading
    } else {
        palette.quote.gamma_multiply(0.7)
    };
    let below = spot.center + vec2(0.0, spot.radius + 5.0);
    let font = FontId::monospace(11.5);
    painter.text(below, Align2::CENTER_TOP, shown, font, colour);
}

/// An arrowhead at the edge of the file a line ends at.
fn arrowhead(painter: &egui::Painter, start: Pos2, end: Pos2, reach: f32, colour: Color32) {
    let way = (end - start).normalized();
    let tip = end - way * reach;
    let back = tip - way * ARROWHEAD.x;
    let side = way.rot90() * ARROWHEAD.y;
    let corners = vec![tip, back + side, back - side];
    painter.add(Shape::convex_polygon(corners, colour, Stroke::NONE));
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::pos2;

    fn area() -> Rect {
        Rect::from_min_size(pos2(100.0, 50.0), vec2(800.0, 600.0))
    }

    #[test]
    fn fitting_keeps_everything_in_view() {
        let bounds = Rect::from_min_max(pos2(-900.0, -100.0), pos2(1500.0, 300.0));
        let camera = Camera::fitted(bounds, area());
        for corner in [bounds.left_top(), bounds.right_bottom()] {
            assert!(area().contains(camera.on_screen(area(), corner)));
        }
    }

    #[test]
    fn zooming_keeps_what_is_under_the_pointer_in_place() {
        let camera = Camera {
            zoom: 1.0,
            pan: vec2(20.0, -10.0),
            fits: true,
        };
        let spot = pos2(300.0, 200.0);
        let place = (spot - area().center() - camera.pan) / camera.zoom;
        let zoomed = camera.zoomed(2.0, spot, area());
        let there = zoomed.on_screen(area(), place.to_pos2());
        assert!((there - spot).length() < 0.01, "{there:?}");
        assert!(!zoomed.fits);
    }

    #[test]
    fn zooming_stops_at_the_ends_of_its_range() {
        let camera = Camera {
            zoom: 1.0,
            pan: Vec2::ZERO,
            fits: false,
        };
        let center = area().center();
        assert!((camera.zoomed(1000.0, center, area()).zoom - ZOOM_RANGE.1).abs() < f32::EPSILON);
        assert!((camera.zoomed(0.0001, center, area()).zoom - ZOOM_RANGE.0).abs() < f32::EPSILON);
    }
}
