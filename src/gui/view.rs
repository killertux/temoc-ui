//! How a page sits in the window: where it is scrolled to and where its wide
//! blocks are, as of the last frame.

use eframe::egui::{Pos2, Rect};

/// How far one press of `j` or `k` scrolls.
const LINE_STEP: f32 = 40.0;
/// How far one press of `Left` or `Right` scrolls.
const SIDEWAYS_STEP: f32 = 64.0;
/// What is too small to count as being scrolled.
const NEARLY_NOTHING: f32 = 0.5;

/// Where something that scrolls is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    Start,
    Middle,
    End,
}

/// How far something is scrolled, and how far it could be.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) struct Reach {
    pub offset: f32,
    /// The offset that shows the end.
    pub last: f32,
}

impl Reach {
    /// How far something shown in `room` is scrolled, if it takes up `size`.
    pub fn of(offset: f32, size: f32, room: f32) -> Self {
        Self {
            offset,
            last: (size - room).max(0.0),
        }
    }

    fn position(self) -> Position {
        if self.offset <= NEARLY_NOTHING {
            Position::Start
        } else if self.offset >= self.last - NEARLY_NOTHING {
            Position::End
        } else {
            Position::Middle
        }
    }

    /// An offset, kept to where there is something to see.
    fn within(self, offset: f32) -> f32 {
        offset.clamp(0.0, self.last)
    }
}

/// A table or code block that scrolls sideways by itself.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Block {
    /// The line of the markdown it starts on, which is how it is known.
    pub line: usize,
    pub rect: Rect,
    pub reach: Reach,
}

/// A way of scrolling the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Scroll {
    LineDown,
    LineUp,
    PageDown,
    PageUp,
    Top,
    Bottom,
}

/// A way of scrolling sideways.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Sideways {
    Left,
    Right,
}

/// How a page was drawn in the last frame.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct View {
    /// Where the page shows.
    pub rect: Rect,
    pub reach: Reach,
    pub blocks: Vec<Block>,
    /// Where each link of the page points, in document order.
    pub links: Vec<String>,
}

impl Default for View {
    fn default() -> Self {
        Self {
            rect: Rect::ZERO,
            reach: Reach::default(),
            blocks: Vec::new(),
            links: Vec::new(),
        }
    }
}

impl View {
    /// Where the page is scrolled to.
    pub fn position(&self) -> Position {
        self.reach.position()
    }

    /// The offset that scrolling this way goes to.
    pub fn scrolled(&self, scroll: Scroll) -> f32 {
        let Reach { offset, last } = self.reach;
        let page = self.rect.height();
        self.reach.within(match scroll {
            Scroll::LineDown => offset + LINE_STEP,
            Scroll::LineUp => offset - LINE_STEP,
            Scroll::PageDown => offset + page,
            Scroll::PageUp => offset - page,
            Scroll::Top => 0.0,
            Scroll::Bottom => last,
        })
    }

    fn block_at_height(&self, height: f32) -> Option<&Block> {
        let mut blocks = self.blocks.iter();
        blocks.find(|block| block.rect.y_range().contains(height))
    }

    /// Where the block at this height is scrolled to, if there is one.
    pub fn sideways_at(&self, height: f32) -> Option<Position> {
        let block = self.block_at_height(height)?;
        Some(block.reach.position())
    }

    /// Which block `Left` and `Right` move: the one under the pointer, or else
    /// the one on `fallback`. Returns its line and the offset to go to.
    pub fn nudged(
        &self,
        pointer: Option<Pos2>,
        fallback: Option<usize>,
        way: Sideways,
    ) -> Option<(usize, f32)> {
        let on_page = pointer.filter(|pointer| self.rect.contains(*pointer));
        let mut blocks = self.blocks.iter();
        let hovered = on_page.and_then(|pointer| {
            let found = blocks.find(|block| block.rect.contains(pointer));
            found.copied()
        });
        let fallback = || {
            let mut blocks = self.blocks.iter();
            let line = fallback?;
            blocks.find(|block| block.line == line).copied()
        };
        let block = hovered.or_else(fallback)?;
        let step = match way {
            Sideways::Left => -SIDEWAYS_STEP,
            Sideways::Right => SIDEWAYS_STEP,
        };
        let offset = block.reach.within(block.reach.offset + step);
        Some((block.line, offset))
    }

    /// The part of this rectangle on screen, if any of it is. What a block has
    /// scrolled away does not show.
    pub fn visible_part(&self, rect: Rect) -> Option<Rect> {
        let block = self.block_at_height(rect.center().y);
        let visible = block.map_or(self.rect, |block| block.rect.intersect(self.rect));
        let part = visible.intersect(rect);
        part.is_positive().then_some(part)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::pos2;

    fn block(line: usize, top: f32, offset: f32, last: f32) -> Block {
        Block {
            line,
            rect: Rect::from_min_max(pos2(0.0, top), pos2(100.0, top + 50.0)),
            reach: Reach { offset, last },
        }
    }

    fn view() -> View {
        View {
            rect: Rect::from_min_max(pos2(0.0, 0.0), pos2(100.0, 200.0)),
            reach: Reach {
                offset: 100.0,
                last: 500.0,
            },
            blocks: vec![block(3, 0.0, 0.0, 300.0), block(9, 60.0, 300.0, 300.0)],
            links: Vec::new(),
        }
    }

    #[test]
    fn nothing_to_scroll_is_at_the_start() {
        assert_eq!(Reach::of(0.0, 50.0, 100.0).position(), Position::Start);
    }

    #[test]
    fn a_scroll_stops_at_either_end() {
        assert!((view().scrolled(Scroll::PageUp)).abs() < f32::EPSILON);
        assert!((view().scrolled(Scroll::LineDown) - 140.0).abs() < f32::EPSILON);
        assert!((view().scrolled(Scroll::Bottom) - 500.0).abs() < f32::EPSILON);
    }

    #[test]
    fn the_pointer_picks_the_block_before_the_focused_table_does() {
        let pointer = Some(pos2(10.0, 10.0));
        let nudged = view().nudged(pointer, Some(9), Sideways::Right);
        assert_eq!(nudged, Some((3, 64.0)));
    }

    #[test]
    fn without_a_block_under_the_pointer_the_focused_table_moves() {
        let pointer = Some(pos2(500.0, 500.0));
        let nudged = view().nudged(pointer, Some(9), Sideways::Left);
        assert_eq!(nudged, Some((9, 236.0)));
    }

    #[test]
    fn without_either_nothing_moves() {
        assert_eq!(view().nudged(None, None, Sideways::Right), None);
    }

    #[test]
    fn a_block_hides_what_it_has_scrolled_out_of_view() {
        let inside = Rect::from_min_max(pos2(10.0, 10.0), pos2(20.0, 20.0));
        let past_the_edge = inside.translate(eframe::egui::vec2(200.0, 0.0));
        assert_eq!(view().visible_part(inside), Some(inside));
        assert_eq!(view().visible_part(past_the_edge), None);
    }
}
