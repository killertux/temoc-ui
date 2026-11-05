//! A test table on the page. See `spec/gui/running.md` and
//! `spec/gui/row-results.md`.
//!
//! The cells scroll sideways. Everything else is placed afterwards around that
//! scroll area, from the rows it reports: the tints behind the rows, the marks
//! in the gutter and the run buttons at the row ends. That keeps them in view
//! however far the cells are scrolled.

use super::{BODY, Drawing, STRONG, Style};
use crate::document::{children, line_of};
use crate::gui::icons::{self, RunLook};
use crate::gui::look::Palette;
use crate::gui::{Command, result_colour, tagged, tags};
use crate::run::{RowResult, TableResult};
use crate::temoc::{Failure, RunResult, TestTable};
use eframe::egui::accesskit::Role;
use eframe::egui::{
    Align, Color32, CornerRadius, FontFamily, FontId, Label, Layout, Pos2, Rect, Response,
    RichText, ScrollArea, Sense, Shape, Stroke, StrokeKind, TextWrapMode, Ui, UiBuilder, Vec2,
    WidgetInfo, pos2, vec2,
};
use markdown::mdast::{Node, Table};

/// How wide the column of marks is.
const GUTTER: f32 = 36.0;
/// How tall a row is without its notes.
const ROW: f32 = 34.0;
/// How far the text of a cell sits from the edges of its column.
const CELL_PADDING: f32 = 12.0;
/// The narrowest a column gets.
const NARROWEST: f32 = 88.0;
/// How far a column grows into spare room. Text that needs more gets more.
const WIDEST: f32 = 170.0;
/// How big a note is.
const NOTE: f32 = 12.5;
/// How big a mark is.
const MARK: f32 = 14.0;
const CORNER: u8 = 6;
const RUN_TABLE: (&str, Vec2) = ("Run table", vec2(100.0, 26.0));
const RUN_ROW: (&str, Vec2) = ("Run row", vec2(86.0, 26.0));
/// How far a run button sits from the end of its row.
const RUN_MARGIN: f32 = 6.0;

/// A row of the table, once the cells are drawn.
struct Placed<'table> {
    /// Where the row is.
    rect: Rect,
    /// The line of its test row. The header has none.
    line: Option<usize>,
    result: Option<&'table RowResult>,
}

impl Drawing<'_> {
    pub(super) fn test_table(
        &mut self,
        ui: &mut Ui,
        table: &Table,
        index: usize,
        test: &TestTable,
    ) {
        let results = self.tests.results;
        let result = results.and_then(|results| results.table(test));
        let spot = ui.scope(|ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
            ui.style_mut().interaction.selectable_labels = false;
            let tints = ui.painter().add(Shape::Noop);
            let left = ui.cursor().left();
            let width = ui.available_width();
            let top = ui.cursor().top();
            let room = width - GUTTER;
            let widths = column_widths(ui, table, room);
            let cells = Cells {
                widths: &widths,
                width: room.max(widths.iter().sum()),
                index,
                test,
                result: result.as_ref(),
            };
            let rows = ui
                .horizontal_top(|ui| {
                    ui.add_space(GUTTER);
                    ScrollArea::horizontal()
                        .id_salt(("test table", test.line))
                        .auto_shrink([false, true])
                        .show(ui, |ui| self.cells(ui, table, &cells))
                        .inner
                })
                .inner;
            let band = |rect: Rect| Rect::from_x_y_ranges(left..=left + width, rect.y_range());
            let mut shapes = Vec::new();
            for row in &rows {
                let row_band = band(row.rect);
                shapes.extend(self.row_tint(ui, row, row_band, index));
                // `put` moves the cursor, which has to stay below the rows.
                let builder = UiBuilder::new()
                    .id_salt(("row ends", row.line))
                    .max_rect(row_band);
                self.row_ends(&mut ui.new_child(builder), row, row_band, &cells);
            }
            if let Some(result) = &result {
                let start = ui.cursor().top();
                self.verdict(ui, result);
                let bottom = ui.cursor().top();
                let footer = Rect::from_x_y_ranges(left..=left + width, start..=bottom);
                let hair = Stroke::new(1.0, self.palette.hair);
                shapes.insert(0, Shape::rect_filled(footer, 0, self.palette.panel));
                shapes.push(Shape::hline(footer.x_range(), start, hair));
            }
            ui.painter().set(tints, Shape::Vec(shapes));
            let outline =
                Rect::from_min_max(pos2(left, top), pos2(left + width, ui.cursor().top()));
            let hair = Stroke::new(1.0, self.palette.hair);
            ui.painter()
                .rect_stroke(outline, CORNER, hair, StrokeKind::Inside);
            self.focused_rect(&rows, index).map_or(outline, band)
        });
        if self.tests.reveal && self.tests.focus.table_index() == Some(index) {
            ui.scroll_to_rect(spot.inner, None);
        }
        ui.add_space(BODY);
    }

    /// The rows of the table, which is all that scrolls sideways.
    fn cells<'results>(
        &mut self,
        ui: &mut Ui,
        table: &Table,
        cells: &Cells<'results, '_>,
    ) -> Vec<Placed<'results>> {
        ui.vertical(|ui| {
            let rows = table.children.iter();
            let mut placed = Vec::new();
            for (position, row) in rows.enumerate() {
                let line = (position > 0).then(|| line_of(row));
                placed.push(self.row(ui, row, line, cells));
            }
            placed
        })
        .inner
    }

    fn row<'results>(
        &mut self,
        ui: &mut Ui,
        row: &Node,
        line: Option<usize>,
        cells: &Cells<'results, '_>,
    ) -> Placed<'results> {
        let result = line.and_then(|line| cells.result?.rows.get(&line));
        let focused = self.is_focused(line, cells.index);
        let style = Style {
            strong: line.is_none(),
            ..Style::default()
        };
        let builder = UiBuilder::new()
            .id_salt(("row", line_of(row)))
            .sense(Sense::click());
        let drawn = ui.scope_builder(builder, |ui| {
            ui.set_min_width(cells.width);
            if focused {
                ui.visuals_mut().override_text_color = Some(self.palette.bar_text);
            }
            ui.horizontal(|ui| {
                for (cell, width) in children(row).iter().zip(cells.widths) {
                    self.cell(ui, cell, *width, style);
                }
            });
            if let (Some(line), Some(result)) = (line, result) {
                self.notes(ui, line, &result.failures, focused);
            }
        });
        let response = drawn.response;
        describe_row(&response, line, cells.test, result, focused);
        if response.clicked() {
            let table = cells.index;
            self.happened.commands.push(Command::Focus { table, line });
        }
        Placed {
            rect: response.rect,
            line,
            result,
        }
    }

    fn cell(&mut self, ui: &mut Ui, cell: &Node, width: f32, style: Style) {
        let size = vec2(width, ROW);
        let layout = Layout::left_to_right(Align::Center);
        ui.allocate_ui_with_layout(size, layout, |ui| {
            ui.set_min_size(size);
            ui.add_space(CELL_PADDING);
            for node in children(cell) {
                self.inline(ui, node, style);
            }
        });
    }

    /// One line below a row for each check that did not pass.
    fn notes(&self, ui: &mut Ui, line: usize, failures: &[Failure], focused: bool) {
        for failure in failures {
            let colour = if focused {
                self.palette.bar_text
            } else {
                note_colour(failure, self.palette)
            };
            let text = failure.note();
            ui.horizontal(|ui| {
                ui.add_space(CELL_PADDING);
                let note = RichText::new(&text).size(NOTE).color(colour);
                let response = ui.add(Label::new(note));
                tagged(&response, tags::NOTE, &format!("{line}: {text}"));
            });
            ui.add_space(8.0);
        }
    }

    /// Whether a row is focused. The header stands for its table.
    fn is_focused(&self, line: Option<usize>, index: usize) -> bool {
        let focus = self.tests.focus;
        match line {
            Some(line) => focus.row(self.tests.tables) == Some(line),
            None => focus.table_index() == Some(index),
        }
    }

    /// What goes behind a row, if anything does.
    fn row_tint(&self, ui: &Ui, row: &Placed<'_>, band: Rect, index: usize) -> Option<Shape> {
        let palette = self.palette;
        let fill = if self.is_focused(row.line, index) {
            palette.bar
        } else if row.line.is_none() {
            palette.raised
        } else {
            let tint = row.result.and_then(|row| tint(row.result, palette));
            let hovered = ui.rect_contains_pointer(band);
            tint.or(hovered.then_some(palette.hover))?
        };
        let corners = if row.line.is_none() {
            CornerRadius {
                nw: CORNER,
                ne: CORNER,
                ..CornerRadius::ZERO
            }
        } else {
            CornerRadius::ZERO
        };
        Some(Shape::rect_filled(band, corners, fill))
    }

    /// What sits at either end of a row, whichever way the cells are scrolled.
    fn row_ends(&mut self, ui: &mut Ui, row: &Placed<'_>, band: Rect, cells: &Cells<'_, '_>) {
        let top = band.top() + (ROW - RUN_TABLE.1.y) / 2.0;
        let right = band.right() - RUN_MARGIN;
        let Some(line) = row.line else {
            self.run_table(ui, pos2(right, top), cells);
            return;
        };
        if let Some(row) = row.result {
            let focused = self.is_focused(Some(line), cells.index);
            self.mark(ui, band, (line, row.result), focused);
        }
        let focused = self.is_focused(Some(line), cells.index);
        if focused || ui.rect_contains_pointer(band) {
            self.run_row(ui, pos2(right, top), line, focused);
        }
    }

    fn mark(&self, ui: &mut Ui, band: Rect, (line, result): (usize, RunResult), focused: bool) {
        let colour = if focused {
            self.palette.bar_text
        } else {
            mark_colour(result, self.palette)
        };
        let gutter = Rect::from_min_size(band.left_top(), vec2(GUTTER, ROW));
        let text = RichText::new(result.mark())
            .font(FontId::new(MARK, FontFamily::Name(STRONG.into())))
            .color(colour);
        let response = ui.put(gutter, Label::new(text));
        tagged(&response, tags::MARK, &format!("{line}: {}", result.mark()));
    }

    fn run_table(&mut self, ui: &mut Ui, top_right: Pos2, cells: &Cells<'_, '_>) {
        let (name, size) = RUN_TABLE;
        let ink = if self.is_focused(None, cells.index) {
            self.palette.bar_text
        } else {
            self.palette.text
        };
        let look = RunLook {
            fill: Color32::TRANSPARENT,
            ink,
        };
        let spot = Rect::from_min_size(top_right - vec2(size.x, 0.0), size);
        let response = self.run_button(ui, spot, name, look);
        describe_button(&response, &cells.test.name);
        if response.clicked() {
            let table = cells.index;
            let commands = &mut self.happened.commands;
            commands.push(Command::Focus { table, line: None });
            commands.push(Command::RunTable);
        }
    }

    fn run_row(&mut self, ui: &mut Ui, top_right: Pos2, line: usize, focused: bool) {
        let (name, size) = RUN_ROW;
        let look = if focused {
            RunLook {
                fill: Color32::TRANSPARENT,
                ink: self.palette.bar_text,
            }
        } else {
            RunLook {
                fill: self.palette.bar,
                ink: self.palette.bar_text,
            }
        };
        let spot = Rect::from_min_size(top_right - vec2(size.x, 0.0), size);
        let response = self.run_button(ui, spot, name, look);
        describe_button(&response, &line.to_string());
        if response.clicked() {
            self.happened.commands.push(Command::RunRow(Some(line)));
        }
    }

    fn run_button(&self, ui: &mut Ui, spot: Rect, name: &str, look: RunLook) -> Response {
        let enabled = !self.tests.busy;
        ui.add_enabled_ui(enabled, |ui| {
            ui.put(spot, |ui: &mut Ui| {
                icons::run_button(ui, spot.size(), name, look)
            })
        })
        .inner
    }

    /// What a table says below itself.
    fn verdict(&self, ui: &mut Ui, table: &TableResult<'_>) {
        let result = table.result;
        let text = format!("{} {result}", result.mark());
        ui.add_space(9.0);
        ui.horizontal(|ui| {
            ui.add_space(CELL_PADDING);
            let verdict = RichText::new(&text)
                .font(FontId::new(13.0, FontFamily::Name(STRONG.into())))
                .color(result_colour(result.name(), self.palette));
            let response = ui.add(Label::new(verdict));
            tagged(&response, tags::VERDICT, &text);
        });
        for failure in table.failures {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(CELL_PADDING);
                let colour = note_colour(failure, self.palette);
                ui.add(Label::new(
                    RichText::new(failure.note()).size(NOTE).color(colour),
                ));
            });
        }
        ui.add_space(9.0);
    }

    /// Where the focus of this table is: its row, or else nothing in particular.
    fn focused_rect(&self, rows: &[Placed<'_>], index: usize) -> Option<Rect> {
        let focus = self.tests.focus;
        let line = focus.row(self.tests.tables)?;
        (focus.table_index() == Some(index))
            .then(|| rows.iter().find(|row| row.line == Some(line)))
            .flatten()
            .map(|row| row.rect)
    }
}

/// What every row of a table goes by.
struct Cells<'results, 'table> {
    widths: &'table [f32],
    /// How wide a row is.
    width: f32,
    /// The position of the table among the tests of the page.
    index: usize,
    test: &'table TestTable,
    result: Option<&'table TableResult<'results>>,
}

/// How wide each column is: wide enough for its text, with spare room shared
/// out from the left.
fn column_widths(ui: &Ui, table: &Table, room: f32) -> Vec<f32> {
    let font = FontId::new(BODY, FontFamily::Name(STRONG.into()));
    let mut widths: Vec<f32> = Vec::new();
    for row in &table.children {
        for (index, cell) in children(row).iter().enumerate() {
            let text =
                ui.painter()
                    .layout_no_wrap(cell.to_string(), font.clone(), Color32::PLACEHOLDER);
            let needed = (text.size().x + 2.0 * CELL_PADDING).max(NARROWEST);
            match widths.get_mut(index) {
                Some(width) => *width = width.max(needed),
                None => widths.push(needed),
            }
        }
    }
    let mut spare = room - widths.iter().sum::<f32>();
    for width in &mut widths {
        let growth = (WIDEST - *width).clamp(0.0, spare.max(0.0));
        *width += growth;
        spare -= growth;
    }
    widths
}

/// Says what a row is, for the accessibility tree.
fn describe_row(
    response: &Response,
    line: Option<usize>,
    test: &TestTable,
    result: Option<&RowResult>,
    focused: bool,
) {
    let name = line.map_or_else(|| test.name.clone(), |line| line.to_string());
    response.widget_info(|| WidgetInfo::labeled(Role::Row, true, &name));
    response.ctx.accesskit_node_builder(response.id, |node| {
        node.set_selected(focused);
        node.set_description(match line {
            Some(_) => tags::TEST_ROW,
            None => tags::TEST_TABLE,
        });
        if let Some(result) = result {
            node.set_value(result.result.name());
        }
    });
}

/// Says what a run button is for, to anyone who cannot see where it sits.
fn describe_button(response: &Response, runs: &str) {
    response.ctx.accesskit_node_builder(response.id, |node| {
        node.set_description(runs);
    });
}

fn note_colour(failure: &Failure, palette: &Palette) -> Color32 {
    match failure.result {
        RunResult::Snoozed => palette.quote,
        _ => palette.note,
    }
}

fn mark_colour(result: RunResult, palette: &Palette) -> Color32 {
    match result {
        RunResult::Skipped | RunResult::Running => palette.quote,
        _ => result_colour(result.name(), palette),
    }
}

/// What a result tints its row with, if anything.
fn tint(result: RunResult, palette: &Palette) -> Option<Color32> {
    match result {
        RunResult::Passed => Some(palette.tint_passed),
        RunResult::Failed | RunResult::Error => Some(palette.tint_failed),
        RunResult::Snoozed => Some(palette.tint_snoozed),
        RunResult::Running => Some(palette.selection),
        RunResult::Skipped | RunResult::Partial => None,
    }
}
