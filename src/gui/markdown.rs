//! Drawing a document on the page.

mod test_table;

use super::look::{ITALIC, Palette, STRONG, STRONG_ITALIC};
use super::view::{Block, Reach};
use super::{Command, tags};
use crate::document::{Document, children, line_of, link_id};
use crate::focus::Focus;
use crate::run::FileResults;
use crate::temoc::TestTable;
use eframe::egui::{
    self, Align, FontFamily, FontId, Frame, Rect, RichText, ScrollArea, Stroke, StrokeKind, Ui,
    vec2,
};
use markdown::mdast::{List, Node, Table};

const BODY: f32 = 14.0;
const CODE: f32 = 13.0;

/// What the page shows of its tests.
#[derive(Debug, Clone, Copy)]
pub(super) struct Tests<'page> {
    /// The tables that can run, in document order.
    pub tables: &'page [TestTable],
    /// What the runs of this file have reported.
    pub results: Option<&'page FileResults>,
    pub focus: Focus,
    /// Whether temoc is running, so that nothing else can start.
    pub busy: bool,
    /// Whether to bring what is focused into view.
    pub reveal: bool,
}

/// What the page has been asked to do in this frame.
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct Wanted {
    /// What tells this visit apart. A visit keeps its own scroll positions.
    pub visit: u64,
    /// The position of the focused link among the links of the page.
    pub focused_link: Option<usize>,
    /// Whether to bring the focused link into view.
    pub reveal_link: bool,
    /// The line of the heading to scroll to.
    pub heading: Option<usize>,
    /// The line of a block to scroll sideways, and the offset to go to.
    pub nudge: Option<(usize, f32)>,
}

/// What happened on the page in this frame.
#[derive(Debug, Default)]
pub(super) struct Happened {
    /// What the clicks on the page ask for.
    pub commands: Vec<Command>,
    /// Where the link under the pointer points.
    pub hovered_link: Option<String>,
    /// Where each link points, in document order.
    pub links: Vec<String>,
    /// The blocks that scroll sideways.
    pub blocks: Vec<Block>,
}

/// Draws a document, and says what happened on it.
pub(super) fn show(
    ui: &mut Ui,
    document: &Document,
    tests: Tests<'_>,
    wanted: Wanted,
    palette: &Palette,
) -> Happened {
    let mut page = Drawing {
        document,
        tests,
        wanted,
        palette,
        happened: Happened::default(),
    };
    for node in children(document.root()) {
        page.block(ui, node);
    }
    page.happened
}

/// A document being drawn.
struct Drawing<'page> {
    document: &'page Document,
    tests: Tests<'page>,
    wanted: Wanted,
    palette: &'page Palette,
    happened: Happened,
}

/// How a run of text is styled.
#[derive(Debug, Clone, Copy, Default)]
struct Style {
    strong: bool,
    emphasis: bool,
    deleted: bool,
}

impl Drawing<'_> {
    fn block(&mut self, ui: &mut Ui, node: &Node) {
        match node {
            Node::Heading(heading) => {
                let size = match heading.depth {
                    1 => 30.0,
                    2 => 21.0,
                    _ => 17.0,
                };
                ui.add_space(size * 0.6);
                let text = RichText::new(node.to_string())
                    .font(FontId::new(size, FontFamily::Name(STRONG.into())))
                    .color(self.palette.heading);
                let response = ui.label(text);
                if self.wanted.heading == Some(line_of(node)) {
                    ui.scroll_to_rect(response.rect, Some(Align::TOP));
                }
                ui.add_space(size * 0.3);
            }
            Node::Paragraph(_) => {
                self.paragraph(ui, children(node));
                ui.add_space(BODY * 0.6);
            }
            Node::List(list) => {
                self.list(ui, list);
                ui.add_space(BODY * 0.6);
            }
            Node::Blockquote(_) => self.quote(ui, node),
            Node::Code(code) => self.code(ui, &code.value, line_of(node)),
            Node::Table(table) => {
                let line = line_of(node);
                let tables = self.tests.tables.iter().enumerate();
                match tables.into_iter().find(|(_, test)| test.line == line) {
                    Some((index, test)) => self.test_table(ui, table, index, test),
                    None => self.table(ui, table, line),
                }
            }
            Node::ThematicBreak(_) => {
                ui.separator();
            }
            Node::Html(html) if !html.value.trim_start().starts_with("<!--") => {
                self.code(ui, &html.value, line_of(node));
            }
            // Directives are definitions, and they draw nothing, like comments.
            _ => {}
        }
    }

    fn paragraph(&mut self, ui: &mut Ui, nodes: &[Node]) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            for node in nodes {
                self.inline(ui, node, Style::default());
            }
        });
    }

    fn inline(&mut self, ui: &mut Ui, node: &Node, style: Style) {
        match node {
            Node::Text(text) => {
                ui.label(styled(&text.value, style));
            }
            Node::InlineCode(code) => {
                let text = RichText::new(&code.value)
                    .font(FontId::monospace(CODE))
                    .color(self.palette.code)
                    .background_color(self.palette.chip);
                ui.label(text);
            }
            Node::Strong(_) => self.inline_all(
                ui,
                node,
                Style {
                    strong: true,
                    ..style
                },
            ),
            Node::Emphasis(_) => self.inline_all(
                ui,
                node,
                Style {
                    emphasis: true,
                    ..style
                },
            ),
            Node::Delete(_) => self.inline_all(
                ui,
                node,
                Style {
                    deleted: true,
                    ..style
                },
            ),
            Node::Link(link) => self.link(ui, node, link.position.as_ref()),
            Node::LinkReference(link) => self.link(ui, node, link.position.as_ref()),
            Node::Break(_) => {
                ui.end_row();
            }
            _ => self.inline_all(ui, node, style),
        }
    }

    fn inline_all(&mut self, ui: &mut Ui, node: &Node, style: Style) {
        for child in children(node) {
            self.inline(ui, child, style);
        }
    }

    fn link(&mut self, ui: &mut Ui, node: &Node, position: Option<&markdown::unist::Position>) {
        let target = self.document.targets().get(&link_id(position));
        let Some(target) = target else {
            // A reference to nothing is only text.
            ui.label(styled(&node.to_string(), Style::default()));
            return;
        };
        // The text colour of the look would win, and a focused row sets its own.
        let on_focused_row = ui.visuals().override_text_color == Some(self.palette.bar_text);
        let colour = if on_focused_row {
            self.palette.bar_text
        } else {
            self.palette.link
        };
        let response = ui.link(RichText::new(node.to_string()).size(BODY).color(colour));
        let focused = self.wanted.focused_link == Some(self.happened.links.len());
        self.happened.links.push(target.url.clone());
        if focused {
            let ring = Stroke::new(1.5, self.palette.bar);
            let rect = response.rect.expand(1.0);
            ui.painter().rect_stroke(rect, 2, ring, StrokeKind::Outside);
            ui.ctx().accesskit_node_builder(response.id, |node| {
                node.set_description(tags::FOCUSED_LINK);
            });
            if self.wanted.reveal_link {
                response.scroll_to_me(None);
            }
        }
        if response.clicked() {
            let follow = Command::Follow(target.url.clone());
            self.happened.commands.push(follow);
        }
        if response.hovered() {
            self.happened.hovered_link = Some(target.url.clone());
        }
    }

    fn list(&mut self, ui: &mut Ui, list: &List) {
        let start = list.start.unwrap_or(1);
        for (item, number) in list.children.iter().zip(start..) {
            ui.horizontal_top(|ui| {
                if list.ordered {
                    ui.label(RichText::new(format!("{number}.")).size(BODY));
                } else {
                    ui.label(RichText::new("•").size(BODY));
                }
                ui.vertical(|ui| {
                    for child in children(item) {
                        match child {
                            // A paragraph in a list needs no room below it.
                            Node::Paragraph(_) => self.paragraph(ui, children(child)),
                            _ => self.block(ui, child),
                        }
                    }
                });
            });
        }
    }

    fn quote(&mut self, ui: &mut Ui, node: &Node) {
        let frame = Frame::new().inner_margin(egui::Margin {
            left: 12,
            ..egui::Margin::ZERO
        });
        let shown = frame.show(ui, |ui| {
            for child in children(node) {
                self.block(ui, child);
            }
        });
        let rect = shown.response.rect;
        let bar = Stroke::new(3.0, self.palette.hair);
        ui.painter().vline(rect.left(), rect.y_range(), bar);
        ui.add_space(BODY * 0.6);
    }

    fn code(&mut self, ui: &mut Ui, text: &str, line: usize) {
        let frame = Frame::new()
            .fill(self.palette.raised)
            .inner_margin(10)
            .corner_radius(6);
        frame.show(ui, |ui| {
            ui.set_width(ui.available_width());
            self.wide_block(ui, line, |_, ui| {
                let text = RichText::new(text).font(FontId::monospace(CODE));
                ui.add(egui::Label::new(text).extend());
            });
        });
        ui.add_space(BODY * 0.6);
    }

    fn table(&mut self, ui: &mut Ui, table: &Table, line: usize) {
        self.wide_block(ui, line, |page, ui| {
            let grid = egui::Grid::new(("table", page.wanted.visit, line)).striped(true);
            grid.show(ui, |ui| {
                for row in &table.children {
                    for cell in children(row) {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            for node in children(cell) {
                                page.inline(ui, node, Style::default());
                            }
                        });
                    }
                    ui.end_row();
                }
            });
        });
        ui.add_space(BODY * 0.6);
    }

    /// Something that scrolls sideways by itself, known by the line it is on.
    fn wide_block<R>(
        &mut self,
        ui: &mut Ui,
        line: usize,
        add: impl FnOnce(&mut Self, &mut Ui) -> R,
    ) -> R {
        let mut area = ScrollArea::horizontal()
            .id_salt(("block", self.wanted.visit, line))
            .auto_shrink([false, true]);
        if let Some((nudged, offset)) = self.wanted.nudge
            && nudged == line
        {
            area = area.horizontal_scroll_offset(offset);
        }
        let shown = area.show(ui, |ui| add(self, ui));
        // `inner_rect` is the room there was, which runs on down the page.
        let height = shown.content_size.y;
        let rect =
            Rect::from_min_size(shown.inner_rect.min, vec2(shown.inner_rect.width(), height));
        let size = shown.content_size.x;
        let reach = Reach::of(shown.state.clamped_offset().x, size, rect.width());
        self.happened.blocks.push(Block { line, rect, reach });
        shown.inner
    }
}

fn styled(text: &str, style: Style) -> RichText {
    let mut text = RichText::new(text).size(BODY);
    let face = match (style.strong, style.emphasis) {
        (false, false) => None,
        (true, false) => Some(STRONG),
        (false, true) => Some(ITALIC),
        (true, true) => Some(STRONG_ITALIC),
    };
    if let Some(face) = face {
        text = text.family(FontFamily::Name(face.into()));
    }
    if style.deleted {
        text = text.strikethrough();
    }
    text
}
