//! What the GUI draws better than the terminal: alerts, task boxes, pictures
//! and footnotes. See `spec/gui/rendering.md`.

use super::{BODY, Drawing, STRONG};
use crate::document::{children, line_of};
use crate::gui::Command;
use crate::gui::look::Palette;
use crate::workspace;
use eframe::egui::accesskit::{Role, Toggled};
use eframe::egui::{
    Button, Color32, CornerRadius, FontFamily, FontId, Frame, Image, Margin, Rect, RichText, Sense,
    Shape, Stroke, StrokeKind, Ui, Vec2, WidgetInfo, pos2, vec2,
};
use markdown::mdast::Node;
use std::time::Duration;

/// How wide and tall a task box is.
const TASK_BOX: f32 = 14.0;
/// How many seconds a code block says that it was copied.
const COPIED_FOR: f64 = 1.5;
/// How many seconds it takes to change what it says.
const COPIED_FADE: f32 = 0.2;

/// Where the definition of a footnote is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Footnote {
    At(usize),
    Missing,
}

/// The kinds of alert a quote can be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Alert {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}

impl Alert {
    fn from_marker(marker: &str) -> Option<Self> {
        match marker {
            "[!NOTE]" => Some(Self::Note),
            "[!TIP]" => Some(Self::Tip),
            "[!IMPORTANT]" => Some(Self::Important),
            "[!WARNING]" => Some(Self::Warning),
            "[!CAUTION]" => Some(Self::Caution),
            _ => None,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Note => "Note",
            Self::Tip => "Tip",
            Self::Important => "Important",
            Self::Warning => "Warning",
            Self::Caution => "Caution",
        }
    }

    /// The colour of its background, and of its title.
    fn colours(self, palette: &Palette) -> (Color32, Color32) {
        match self {
            Self::Note | Self::Tip => (palette.selection, palette.link),
            Self::Important => (palette.selection, palette.code),
            Self::Warning => (palette.tint_snoozed, palette.snoozed),
            Self::Caution => (palette.tint_failed, palette.failed),
        }
    }
}

/// What a quote starting with an alert marker says: its kind, and its blocks
/// without the marker.
pub(super) fn alert_of(quote: &Node) -> Option<(Alert, Vec<Node>)> {
    let first = children(quote).first()?;
    let Some(Node::Text(text)) = children(first).first() else {
        return None;
    };
    let (marker, rest) = text.value.split_once('\n').unwrap_or((&text.value, ""));
    let alert = Alert::from_marker(marker.trim())?;
    let mut blocks = children(quote).to_vec();
    if let Some(Node::Paragraph(paragraph)) = blocks.first_mut() {
        if rest.is_empty() {
            paragraph.children.remove(0);
        } else if let Some(Node::Text(text)) = paragraph.children.first_mut() {
            rest.clone_into(&mut text.value);
        }
    }
    Some((alert, blocks))
}

impl Drawing<'_> {
    pub(super) fn alert(&mut self, ui: &mut Ui, alert: Alert, blocks: &[Node]) {
        let (fill, ink) = alert.colours(self.palette);
        // The last block leaves room below itself.
        let margin = Margin {
            bottom: 0,
            ..box_margin()
        };
        let frame = Frame::new()
            .fill(fill)
            .corner_radius(6)
            .inner_margin(margin);
        frame.show(ui, |ui| {
            ui.set_width(ui.available_width());
            let font = FontId::new(13.0, FontFamily::Name(STRONG.into()));
            ui.label(RichText::new(alert.title()).font(font).color(ink));
            for block in blocks {
                self.block(ui, block);
            }
        });
        ui.add_space(BODY * 0.6);
    }

    /// The box of a task list item, which only shows what the file says.
    pub(super) fn task_box(&self, ui: &mut Ui, size: Vec2, checked: bool, name: &str) {
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        response.widget_info(|| WidgetInfo::labeled(Role::CheckBox, true, name));
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_toggled(if checked {
                Toggled::True
            } else {
                Toggled::False
            });
        });
        let spot = Rect::from_center_size(
            pos2(rect.left() + TASK_BOX / 2.0, rect.center().y),
            Vec2::splat(TASK_BOX),
        );
        let painter = ui.painter();
        let line = self.palette.line;
        if checked {
            painter.rect_filled(spot, 3, line);
            let tick = vec![
                spot.left_center() + vec2(3.0, 0.0),
                spot.center_bottom() + vec2(-1.0, -4.0),
                spot.right_top() + vec2(-3.0, 4.0),
            ];
            let ink = Stroke::new(1.5, self.palette.page);
            painter.add(Shape::line(tick, ink));
        } else {
            painter.rect_stroke(spot, 3, Stroke::new(1.0, line), StrokeKind::Inside);
        }
    }

    /// A picture of the workspace, or else what it was meant to be.
    pub(super) fn picture(&mut self, ui: &mut Ui, url: &str, alt: &str) {
        let path = workspace::picture_path(self.workspace, self.file, url);
        let ctx = ui.ctx().clone();
        let picture = path.and_then(|path| self.pictures.get(&ctx, &path));
        let Some(picture) = picture else {
            self.missing_picture(ui, url, alt);
            return;
        };
        let size = picture.size_vec2();
        let scale = (self.column / size.x).min(1.0);
        let response = ui.add(Image::new((picture.id(), size * scale)));
        // The address is its name. The alt text does not show, but it is what
        // a screen reader has to go by.
        response.widget_info(|| WidgetInfo::labeled(Role::Image, true, url));
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_description(alt);
        });
    }

    fn missing_picture(&self, ui: &mut Ui, url: &str, alt: &str) {
        let quiet = self.palette.quote;
        let shown = Frame::new().inner_margin(box_margin()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(alt).size(13.0).color(quiet));
                ui.add_space(8.0);
                ui.label(
                    RichText::new(url)
                        .font(FontId::monospace(12.0))
                        .color(quiet),
                );
            });
        });
        let rect = shown.response.rect;
        let corners = [
            rect.left_top(),
            rect.right_top(),
            rect.right_bottom(),
            rect.left_bottom(),
            rect.left_top(),
        ];
        let line = Stroke::new(1.0, self.palette.line);
        let dashes = Shape::dashed_line(&corners, line, 4.0, 3.0);
        ui.painter().extend(dashes);
    }

    /// The button that copies a code block, which says so for a moment.
    pub(super) fn copy_button(&mut self, ui: &mut Ui, text: &str, line: usize) {
        let id = ui.scope_id().with(("copied", self.wanted.visit, line));
        let now = ui.input(|input| input.time);
        let copied_at = ui.data(|data| data.get_temp::<f64>(id));
        let left = copied_at.map_or(0.0, |copied_at| copied_at + COPIED_FOR - now);
        let fade = ui.ctx().animate_bool_with_time(id, left > 0.0, COPIED_FADE);
        // One word fades out, and then the other fades in.
        let (word, ink) = if fade > 0.5 {
            ("✓ Copied", self.palette.passed)
        } else {
            ("Copy", self.palette.quote)
        };
        let ink = ink.gamma_multiply((2.0 * fade - 1.0).abs());
        let word = RichText::new(word).size(12.0).color(ink);
        if ui.add(Button::new(word).frame(false)).clicked() {
            ui.data_mut(|data| data.insert_temp(id, now));
            self.happened.commands.push(Command::Copy(text.to_owned()));
        }
        if let Ok(wait) = Duration::try_from_secs_f64(left) {
            ui.ctx().request_repaint_after(wait);
        }
    }

    /// Where the definition of a footnote is.
    pub(super) fn footnote_line(&self, identifier: &str) -> Footnote {
        let definitions = children(self.document.root()).iter();
        let definition = definitions.into_iter().find(|node| {
            matches!(node, Node::FootnoteDefinition(definition)
                if definition.identifier == identifier)
        });
        definition.map_or(Footnote::Missing, |node| Footnote::At(line_of(node)))
    }

    /// The text of a footnote, after its number.
    pub(super) fn footnote(&mut self, ui: &mut Ui, identifier: &str, blocks: &[Node]) {
        ui.horizontal_top(|ui| {
            ui.label(
                RichText::new(identifier)
                    .size(12.0)
                    .color(self.palette.quote),
            );
            ui.vertical(|ui| {
                for block in blocks {
                    self.block(ui, block);
                }
            });
        });
    }
}

/// How far the text of a box sits from its edges.
fn box_margin() -> Margin {
    Margin::symmetric(14, 8)
}

/// The corners a row keeps when it is the top of a frame.
pub(super) fn top_corners(radius: u8) -> CornerRadius {
    CornerRadius {
        nw: radius,
        ne: radius,
        ..CornerRadius::ZERO
    }
}
