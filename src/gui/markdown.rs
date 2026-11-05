//! Drawing a document on the page.

mod extras;
mod test_table;

use super::look::{ITALIC, Palette, STRONG, STRONG_ITALIC};
use super::pictures::Pictures;
use super::view::{Block, Reach};
use super::{Command, is_website, tags};
use crate::document::{Document, children, line_of, link_id};
use crate::focus::Focus;
use crate::run::FileResults;
use crate::temoc::TestTable;
use eframe::egui::{
    self, Align, FontFamily, FontId, Frame, Margin, Rect, Response, RichText, ScrollArea, Sense,
    Shape, Stroke, StrokeKind, TextWrapMode, Ui, UiBuilder, Vec2, vec2,
};
use markdown::mdast::{AlignKind, List, Node, Table};
use std::path::Path;

const BODY: f32 = 14.0;
const CODE: f32 = 13.0;
/// The widest the text of a page gets.
const COLUMN: f32 = 720.0;
/// How far each level of a list moves in.
const LIST_INDENT: f32 = 24.0;
/// How tall a line of a list is.
const LIST_LINE: f32 = 20.0;
/// How tall a row of a table is.
const TABLE_ROW: f32 = 30.0;
/// The size of each level of heading.
const HEADINGS: [f32; 6] = [30.0, 21.0, 17.0, 15.0, 14.0, 13.0];

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
    /// The line of the heading or footnote to scroll to.
    pub go_to: Option<usize>,
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
    /// What following each link does, in document order.
    pub links: Vec<Command>,
    /// The blocks that scroll sideways.
    pub blocks: Vec<Block>,
}

/// What is on the page.
#[derive(Debug, Clone, Copy)]
pub(super) struct Page<'page> {
    pub document: &'page Document,
    /// The file it comes from, relative to the workspace.
    pub file: &'page str,
    pub workspace: &'page Path,
    pub tests: Tests<'page>,
    pub wanted: Wanted,
}

/// Draws a document in a column down the middle, and says what happened on it.
pub(super) fn show(
    ui: &mut Ui,
    page: Page<'_>,
    pictures: &mut Pictures,
    palette: &Palette,
) -> Happened {
    let room = ui.available_width();
    let column = room.min(COLUMN);
    let side = ((room - column) / 2.0).max(0.0);
    let rect = Rect::from_min_size(
        ui.cursor().min + vec2(side, 0.0),
        vec2(column, ui.available_height()),
    );
    let mut drawing = Drawing {
        document: page.document,
        file: page.file,
        workspace: page.workspace,
        tests: page.tests,
        wanted: page.wanted,
        pictures,
        palette,
        column,
        list_depth: 0,
        happened: Happened::default(),
    };
    ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
        for node in children(page.document.root()) {
            drawing.block(ui, node);
        }
    });
    drawing.happened
}

/// A document being drawn.
struct Drawing<'page> {
    document: &'page Document,
    file: &'page str,
    workspace: &'page Path,
    tests: Tests<'page>,
    wanted: Wanted,
    pictures: &'page mut Pictures,
    palette: &'page Palette,
    /// How wide the text of the page is.
    column: f32,
    /// How many lists the list being drawn is inside of.
    list_depth: usize,
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
            Node::Heading(heading) => self.heading(ui, node, heading.depth),
            Node::Paragraph(_) => {
                self.paragraph(ui, children(node));
                ui.add_space(BODY * 0.6);
            }
            Node::List(list) => {
                self.list(ui, list);
                ui.add_space(BODY * 0.6);
            }
            Node::Blockquote(_) => match extras::alert_of(node) {
                Some((alert, blocks)) => self.alert(ui, alert, &blocks),
                None => self.quote(ui, node),
            },
            Node::Code(code) => {
                self.code(ui, &code.value, code.lang.as_deref(), line_of(node));
            }
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
            Node::Html(html) if !is_comment(&html.value) => {
                let text = RichText::new(&html.value).font(FontId::monospace(CODE));
                ui.label(text.color(self.palette.quote));
                ui.add_space(BODY * 0.6);
            }
            Node::FootnoteDefinition(footnote) => {
                let top = ui.cursor().top();
                self.footnote(ui, &footnote.identifier, &footnote.children);
                self.arrive(ui, node, top);
            }
            _ => {}
        }
    }

    fn heading(&self, ui: &mut Ui, node: &Node, depth: u8) {
        let level = usize::from(depth).saturating_sub(1);
        let size = HEADINGS.get(level).copied().unwrap_or(CODE);
        // The last level is quiet, and only its size tells it from the text.
        let (family, colour) = if usize::from(depth) >= HEADINGS.len() {
            (FontFamily::Proportional, self.palette.quote)
        } else {
            (FontFamily::Name(STRONG.into()), self.palette.heading)
        };
        ui.add_space(size * 0.6);
        let top = ui.cursor().top();
        let text = RichText::new(node.to_string()).font(FontId::new(size, family));
        ui.label(text.color(colour));
        self.arrive(ui, node, top);
        ui.add_space(size * 0.3);
    }

    /// Scrolls to something if the page was asked to go there.
    fn arrive(&self, ui: &Ui, node: &Node, top: f32) {
        if self.wanted.go_to == Some(line_of(node)) {
            let rect = Rect::from_x_y_ranges(ui.max_rect().x_range(), top..=ui.cursor().top());
            ui.scroll_to_rect(rect, Some(Align::TOP));
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
                // A line break in the markdown is only a space on the page.
                ui.label(styled(&text.value.replace('\n', " "), style));
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
            Node::Image(image) => self.picture(ui, &image.url, &image.alt),
            Node::ImageReference(image) => {
                let url = self.document.definition(&image.identifier);
                self.picture(ui, url.unwrap_or_default(), &image.alt);
            }
            Node::FootnoteReference(footnote) => self.footnote_reference(ui, footnote),
            Node::Html(html) if !is_comment(&html.value) => {
                ui.label(RichText::new(&html.value).color(self.palette.quote));
            }
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
        let url = &target.url;
        let text = RichText::new(node.to_string()).size(BODY).underline();
        let response = self.link_to(ui, text.color(colour), Command::Follow(url.clone()));
        if response.hovered() {
            self.happened.hovered_link = Some(url.clone());
        }
        if is_website(url) {
            // Says that the link leaves the app.
            ui.label(RichText::new(" ↗").size(BODY).color(colour));
        }
    }

    /// Something that can be focused and followed.
    fn link_to(&mut self, ui: &mut Ui, text: RichText, follow: Command) -> Response {
        let response = ui.link(text);
        let focused = self.wanted.focused_link == Some(self.happened.links.len());
        self.happened.links.push(follow.clone());
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
            self.happened.commands.push(follow);
        }
        response
    }

    fn list(&mut self, ui: &mut Ui, list: &List) {
        let start = list.start.unwrap_or(1);
        self.list_depth = self.list_depth.saturating_add(1);
        for (item, number) in list.children.iter().zip(start..) {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                let marker = vec2(LIST_INDENT, LIST_LINE);
                match item {
                    Node::ListItem(task) if task.checked.is_some() => {
                        let checked = task.checked == Some(true);
                        self.task_box(ui, marker, checked, &item.to_string());
                    }
                    _ if list.ordered => {
                        let text = RichText::new(format!("{number}.")).size(BODY);
                        let layout = egui::Layout::left_to_right(Align::Center);
                        let colour = self.palette.quote;
                        ui.allocate_ui_with_layout(marker, layout, |ui| {
                            ui.set_min_size(marker);
                            ui.label(text.color(colour));
                        });
                    }
                    _ => self.bullet(ui, marker),
                }
                ui.vertical(|ui| {
                    for child in children(item) {
                        match child {
                            Node::Paragraph(_) => self.paragraph(ui, children(child)),
                            _ => self.block(ui, child),
                        }
                    }
                });
            });
        }
        self.list_depth = self.list_depth.saturating_sub(1);
    }

    /// The marker of a list item, which changes with each level.
    fn bullet(&self, ui: &mut Ui, size: Vec2) {
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        let spot = egui::pos2(rect.left() + 5.0, rect.center().y);
        let painter = ui.painter();
        let colour = self.palette.quote;
        match self.list_depth {
            0 | 1 => {
                painter.circle_filled(spot, 2.0, colour);
            }
            2 => {
                painter.circle_stroke(spot, 2.0, Stroke::new(1.0, colour));
            }
            _ => {
                let square = Rect::from_center_size(spot, Vec2::splat(4.0));
                painter.rect_filled(square, 0, colour);
            }
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
        // The last block leaves room below itself, which the bar stops short of.
        let rect = shown.response.rect;
        let bar = Stroke::new(3.0, self.palette.hair);
        let bottom = (rect.bottom() - BODY * 0.6).max(rect.top());
        ui.painter().vline(rect.left(), rect.top()..=bottom, bar);
        ui.add_space(BODY * 0.6);
    }

    fn code(&mut self, ui: &mut Ui, text: &str, language: Option<&str>, line: usize) {
        let hair = Stroke::new(1.0, self.palette.hair);
        let frame = Frame::new()
            .fill(self.palette.panel)
            .stroke(hair)
            .corner_radius(6);
        frame.show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.set_width(ui.available_width());
            let header = Frame::new().inner_margin(Margin::symmetric(14, 2));
            let shown = header.show(ui, |ui| {
                ui.horizontal(|ui| {
                    let quiet = self.palette.quote;
                    if let Some(language) = language {
                        let name = RichText::new(language).font(FontId::monospace(12.0));
                        ui.label(name.color(quiet));
                    }
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        let copy = RichText::new("Copy").size(12.0).color(quiet);
                        if ui.add(egui::Button::new(copy).frame(false)).clicked() {
                            let copy = Command::Copy(text.to_owned());
                            self.happened.commands.push(copy);
                        }
                    });
                });
            });
            let bottom = shown.response.rect.bottom();
            ui.painter().hline(ui.max_rect().x_range(), bottom, hair);
            let body = Frame::new().inner_margin(Margin::symmetric(14, 12));
            body.show(ui, |ui| {
                self.wide_block(ui, line, |_, ui| {
                    let shown = text.replace('\t', "    ");
                    let shown = RichText::new(shown).font(FontId::monospace(CODE));
                    ui.add(egui::Label::new(shown).extend());
                });
            });
        });
        ui.add_space(BODY);
    }

    fn table(&mut self, ui: &mut Ui, table: &Table, line: usize) {
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
            let tints = ui.painter().add(Shape::Noop);
            let left = ui.cursor().left();
            let top = ui.cursor().top();
            let width = ui.available_width();
            let widths = test_table::column_widths(ui, table, width);
            let rows = self.wide_block(ui, line, |page, ui| {
                ui.vertical(|ui| {
                    let mut bottoms = Vec::new();
                    for (position, row) in table.children.iter().enumerate() {
                        let style = Style {
                            strong: position == 0,
                            ..Style::default()
                        };
                        let drawn = ui.horizontal(|ui| {
                            let cells = children(row).iter().zip(&widths);
                            for (column, (cell, width)) in cells.enumerate() {
                                let align = table.align.get(column).copied();
                                let spot = test_table::Spot {
                                    width: *width,
                                    height: TABLE_ROW,
                                    align: align.unwrap_or(AlignKind::None),
                                };
                                page.cell(ui, cell, spot, style);
                            }
                        });
                        bottoms.push(drawn.response.rect.bottom());
                    }
                    bottoms
                })
                .inner
            });
            let hair = Stroke::new(1.0, self.palette.hair);
            let span = left..=left + width;
            let mut shapes = Vec::new();
            if let Some(header) = rows.first() {
                let rect = Rect::from_x_y_ranges(span.clone(), top..=*header);
                let corners = extras::top_corners(6);
                shapes.push(Shape::rect_filled(rect, corners, self.palette.raised));
            }
            let lines = rows.iter().rev().skip(1);
            shapes.extend(lines.map(|bottom| Shape::hline(span.clone(), *bottom, hair)));
            ui.painter().set(tints, Shape::Vec(shapes));
            let bottom = ui.cursor().top();
            let outline = Rect::from_x_y_ranges(span, top..=bottom);
            ui.painter()
                .rect_stroke(outline, 6, hair, StrokeKind::Inside);
        });
        ui.add_space(BODY);
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

fn is_comment(html: &str) -> bool {
    html.trim_start().starts_with("<!--")
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
