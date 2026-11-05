//! Text inside a block. Each paragraph is one piece of text, so that it wraps
//! as a whole, and what is selected in it is copied the way it shows.

use super::extras::Footnote;
use super::{BODY, CODE, Drawing, Style, is_comment};
use crate::document::{children, link_id};
use crate::gui::look::{ITALIC, STRONG, STRONG_ITALIC};
use crate::gui::{Command, is_website, tags};
use eframe::egui::accesskit::Role;
use eframe::egui::text::{CCursor, LayoutJob, TextFormat};
use eframe::egui::{
    Align, Color32, CursorIcon, FontFamily, FontId, Galley, Label, Pos2, Rect, Response, Sense,
    Shape, Stroke, StrokeKind, TextWrapMode, Ui, WidgetInfo,
};
use markdown::mdast::Node;
use std::ops::Range;
use std::sync::Arc;

/// How far below the middle of its line the line under a link is.
const UNDERLINE: f32 = 9.0;

/// Something in a piece of text that can be followed.
#[derive(Debug)]
struct Link {
    /// The text it goes by.
    name: String,
    /// Which characters of the piece of text it covers.
    characters: Range<usize>,
    follow: Command,
    /// Where it points, to say so when the pointer is over it.
    target: Option<String>,
    /// The line under it, if it has one.
    underline: Option<Underline>,
}

/// The line under a link.
#[derive(Debug, Clone, Copy)]
struct Underline {
    colour: Color32,
    /// Whether it is dashed, which says that the link leads nowhere.
    dashed: bool,
}

/// A piece of text being put together.
#[derive(Debug, Default)]
struct Piece {
    job: LayoutJob,
    /// How many characters it has so far.
    length: usize,
    /// How tall its lines are, if not as tall as their font.
    line: Option<f32>,
    links: Vec<Link>,
}

impl Piece {
    /// Adds some text that takes the height of the line.
    fn push(&mut self, text: &str, mut format: TextFormat) {
        format.line_height = self.line;
        self.add(text, format);
    }

    /// Adds some text as it is, and says which characters it became.
    fn add(&mut self, text: &str, format: TextFormat) -> Range<usize> {
        let start = self.length;
        self.length = start.saturating_add(text.chars().count());
        self.job.append(text, 0.0, format);
        start..self.length
    }

    fn link(&mut self, format: TextFormat, link: Link) {
        let characters = self.add(&link.name, format);
        self.links.push(Link { characters, ..link });
    }
}

impl Drawing<'_> {
    /// Draws text and whatever sits in it, in lines of this height if one is
    /// given.
    pub(super) fn inline(&mut self, ui: &mut Ui, nodes: &[Node], style: Style, line: Option<f32>) {
        let mut piece = Piece {
            line,
            ..Piece::default()
        };
        for node in nodes {
            self.gather(ui, &mut piece, node, style);
        }
        self.finish(ui, &mut piece);
    }

    fn gather(&mut self, ui: &mut Ui, piece: &mut Piece, node: &Node, style: Style) {
        match node {
            Node::Text(text) => {
                // A line break in the markdown is only a space on the page.
                let shown = text.value.replace('\n', " ");
                piece.push(&shown, format(ui, style));
            }
            Node::InlineCode(code) => {
                // Its background is as tall as its text, and not as its line.
                let format = TextFormat {
                    font_id: FontId::monospace(CODE),
                    color: self.palette.code,
                    background: self.palette.chip,
                    valign: Align::Center,
                    ..TextFormat::default()
                };
                piece.add(&code.value, format);
            }
            Node::Strong(_) => self.gather_all(
                ui,
                piece,
                node,
                Style {
                    strong: true,
                    ..style
                },
            ),
            Node::Emphasis(_) => self.gather_all(
                ui,
                piece,
                node,
                Style {
                    emphasis: true,
                    ..style
                },
            ),
            Node::Delete(_) => self.gather_all(
                ui,
                piece,
                node,
                Style {
                    deleted: true,
                    ..style
                },
            ),
            Node::Link(link) => self.gather_link(ui, piece, node, link.position.as_ref()),
            Node::LinkReference(link) => self.gather_link(ui, piece, node, link.position.as_ref()),
            Node::Image(image) => {
                self.finish(ui, piece);
                self.picture(ui, &image.url, &image.alt);
            }
            Node::ImageReference(image) => {
                self.finish(ui, piece);
                let url = self.document.definition(&image.identifier);
                self.picture(ui, url.unwrap_or_default(), &image.alt);
            }
            Node::FootnoteReference(footnote) => {
                self.gather_footnote(ui, piece, &footnote.identifier);
            }
            Node::Html(html) if !is_comment(&html.value) => {
                let format = TextFormat {
                    color: self.palette.quote,
                    ..format(ui, style)
                };
                piece.push(&html.value, format);
            }
            Node::Html(_) => {}
            Node::Break(_) => {
                piece.push("\n", format(ui, style));
            }
            _ => self.gather_all(ui, piece, node, style),
        }
    }

    fn gather_all(&mut self, ui: &mut Ui, piece: &mut Piece, node: &Node, style: Style) {
        for child in children(node) {
            self.gather(ui, piece, child, style);
        }
    }

    fn gather_link(
        &self,
        ui: &Ui,
        piece: &mut Piece,
        node: &Node,
        position: Option<&markdown::unist::Position>,
    ) {
        let name = node.to_string();
        let Some(target) = self.document.targets().get(&link_id(position)) else {
            // A reference to nothing is only text.
            piece.push(&name, format(ui, Style::default()));
            return;
        };
        // The text colour of the look would win, and a focused row sets its own.
        let on_focused_row = ui.visuals().override_text_color == Some(self.palette.bar_text);
        let url = &target.url;
        let dashed = self.missing.contains(url);
        let colour = if on_focused_row {
            self.palette.bar_text
        } else if dashed {
            self.palette.quote
        } else {
            self.palette.link
        };
        let plain = TextFormat {
            color: colour,
            line_height: piece.line,
            ..format(ui, Style::default())
        };
        let link = Link {
            name,
            characters: 0..0,
            follow: Command::Follow(url.clone()),
            target: Some(url.clone()),
            underline: Some(Underline { colour, dashed }),
        };
        piece.link(plain.clone(), link);
        if is_website(url) {
            // Says that the link leaves the app.
            piece.push(" ↗", plain);
        }
    }

    fn gather_footnote(&self, ui: &Ui, piece: &mut Piece, identifier: &str) {
        // It only sits high if it is not as tall as its line.
        let plain = TextFormat {
            font_id: FontId::proportional(10.0),
            valign: Align::TOP,
            ..format(ui, Style::default())
        };
        match self.footnote_line(identifier) {
            Footnote::At(definition) => {
                let format = TextFormat {
                    color: self.palette.link,
                    ..plain
                };
                let reference = Link {
                    name: identifier.to_owned(),
                    characters: 0..0,
                    follow: Command::GoToLine(definition),
                    target: None,
                    underline: None,
                };
                piece.link(format, reference);
            }
            Footnote::Missing => {
                piece.add(identifier, plain);
            }
        }
    }

    /// Draws what has been put together so far.
    fn finish(&mut self, ui: &mut Ui, piece: &mut Piece) {
        let line = piece.line;
        let Piece { mut job, links, .. } = std::mem::take(piece);
        piece.line = line;
        if job.text.is_empty() {
            return;
        }
        job.first_row_min_height = line.unwrap_or_default();
        job.wrap.max_width = match ui.wrap_mode() {
            TextWrapMode::Extend => f32::INFINITY,
            TextWrapMode::Wrap | TextWrapMode::Truncate => ui.available_width(),
        };
        let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
        let response = ui.add(Label::new(Arc::clone(&galley)));
        let mut followed = false;
        for link in links {
            let rects = rects_of(&galley, &link.characters, response.rect.min);
            followed |= self.link(ui, &response, link, &rects);
        }
        self.clicked_text |= response.clicked() && !followed;
    }

    /// Hit tests a link against the text it is in, and says if it was followed.
    /// The text keeps the pointer, so that dragging from a link selects.
    fn link(&mut self, ui: &Ui, text: &Response, link: Link, rects: &[Rect]) -> bool {
        let over = |spot: Option<Pos2>| {
            spot.is_some_and(|spot| rects.iter().any(|rect| rect.contains(spot)))
        };
        let focused = self.wanted.focused_link == Some(self.happened.links.len());
        self.happened.links.push(link.follow.clone());
        if let Some(first) = rects.first() {
            // Only there to be found: it has no use for the pointer.
            let id = text.id.with(link.characters.start);
            let node = ui.interact(*first, id, Sense::hover());
            node.widget_info(|| WidgetInfo::labeled(Role::Link, true, &link.name));
            if link.underline.is_some_and(|underline| underline.dashed) {
                ui.ctx().accesskit_node_builder(node.id, |node| {
                    node.set_description(tags::MISSING_LINK);
                });
            }
            if focused {
                ui.ctx().accesskit_node_builder(node.id, |node| {
                    node.set_description(tags::FOCUSED_LINK);
                });
                if self.wanted.reveal_link {
                    ui.scroll_to_rect(*first, None);
                }
            }
        }
        if let Some(underline) = link.underline {
            for rect in rects {
                paint_underline(ui, *rect, underline);
            }
        }
        if focused {
            let ring = Stroke::new(1.5, self.palette.bar);
            for rect in rects {
                let rect = rect.expand(1.0);
                ui.painter().rect_stroke(rect, 2, ring, StrokeKind::Outside);
            }
        }
        if over(text.hover_pos()) {
            ui.set_cursor_icon(CursorIcon::PointingHand);
            if link.target.is_some() {
                self.happened.hovered_link = link.target;
            }
        }
        let followed = text.clicked() && over(text.interact_pointer_pos());
        if followed {
            self.happened.commands.push(link.follow);
        }
        followed
    }
}

/// Draws the line under part of a link. The text would draw it at the bottom of
/// the line it is on, far below itself.
fn paint_underline(ui: &Ui, rect: Rect, underline: Underline) {
    let stroke = Stroke::new(1.0, underline.colour);
    let height = rect.center().y + UNDERLINE;
    if underline.dashed {
        let ends = [
            Pos2::new(rect.left(), height),
            Pos2::new(rect.right(), height),
        ];
        let dashes = Shape::dashed_line(&ends, stroke, 3.0, 2.0);
        ui.painter().extend(dashes);
    } else {
        ui.painter().hline(rect.x_range(), height, stroke);
    }
}

/// How text of this style looks.
fn format(ui: &Ui, style: Style) -> TextFormat {
    let family = match (style.strong, style.emphasis) {
        (false, false) => FontFamily::Proportional,
        (true, false) => FontFamily::Name(STRONG.into()),
        (false, true) => FontFamily::Name(ITALIC.into()),
        (true, true) => FontFamily::Name(STRONG_ITALIC.into()),
    };
    let color = ui.visuals().text_color();
    let strikethrough = if style.deleted {
        Stroke::new(1.0, color)
    } else {
        Stroke::NONE
    };
    TextFormat {
        font_id: FontId::new(BODY, family),
        color,
        strikethrough,
        ..TextFormat::default()
    }
}

/// Where some characters of a text are, with one rectangle for each line they
/// are on.
fn rects_of(galley: &Galley, characters: &Range<usize>, origin: Pos2) -> Vec<Rect> {
    let mut rects: Vec<Rect> = Vec::new();
    for index in characters.start..=characters.end {
        let spot = galley
            .pos_from_cursor(CCursor::new(index))
            .translate(origin.to_vec2());
        match rects.last_mut() {
            Some(rect) if rect.y_range().contains(spot.center().y) => rect.max.x = spot.right(),
            _ => rects.push(spot),
        }
    }
    rects
}
