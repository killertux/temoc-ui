//! The toolbar's icons. The design draws them with lines, so that no icon font
//! is needed.

use eframe::egui::accesskit::Role;
use eframe::egui::{
    CornerRadius, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui, Vec2,
    WidgetInfo, pos2, vec2,
};

/// How wide and tall an icon is.
const SIZE: f32 = 16.0;
/// How wide and tall a button holding an icon is.
const BUTTON: f32 = 28.0;
/// How thick the lines of an icon are.
const LINE: f32 = 1.5;

/// One of the icons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Icon {
    LeftPanel,
    RightPanel,
    Back,
    Forward,
    Contrast,
}

/// A button showing an icon, which goes by `name` everywhere else.
pub(super) fn button(ui: &mut Ui, icon: Icon, name: &str) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(BUTTON), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, name));
    let visuals = ui.style().interact(&response);
    if response.hovered() {
        let painter = ui.painter();
        painter.rect_filled(rect, CornerRadius::same(6), visuals.bg_fill);
    }
    let stroke = Stroke::new(LINE, visuals.fg_stroke.color);
    draw(ui.painter(), icon, rect.center(), stroke);
    response.on_hover_text(name)
}

fn draw(painter: &Painter, icon: Icon, center: Pos2, stroke: Stroke) {
    let area = Rect::from_center_size(center, Vec2::splat(SIZE));
    let chevron = |tip: f32, arms: f32| {
        let points = vec![
            pos2(center.x + arms, center.y - 5.0),
            pos2(center.x + tip, center.y),
            pos2(center.x + arms, center.y + 5.0),
        ];
        painter.add(Shape::line(points, stroke));
    };
    match icon {
        Icon::LeftPanel => panel(painter, area, area.left() + 5.5, stroke),
        Icon::RightPanel => panel(painter, area, area.right() - 5.5, stroke),
        Icon::Back => chevron(-3.0, 2.5),
        Icon::Forward => chevron(3.0, -2.5),
        Icon::Contrast => {
            let radius = SIZE / 2.0 - 1.0;
            painter.circle_stroke(center, radius, stroke);
            let half = (0_i16..=16).map(|step| {
                let angle = std::f32::consts::PI * (f32::from(step) / 16.0 - 0.5);
                center + radius * vec2(angle.cos(), angle.sin())
            });
            painter.add(Shape::convex_polygon(
                half.collect(),
                stroke.color,
                Stroke::NONE,
            ));
        }
    }
}

/// A window with a line down it, where its panel ends.
fn panel(painter: &Painter, area: Rect, line: f32, stroke: Stroke) {
    let window = area.shrink2(vec2(0.5, 1.5));
    painter.rect_stroke(window, CornerRadius::same(2), stroke, StrokeKind::Middle);
    painter.vline(line, window.y_range(), stroke);
}
