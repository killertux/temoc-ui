//! Finding text on the rendered screen.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};

/// Where a piece of text is on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Found {
    pub first: (u16, u16),
    pub last: (u16, u16),
}

/// Finds the first occurrence of `text` in `area`, searching from the top.
pub fn find(buffer: &Buffer, area: Rect, text: &str) -> Option<Found> {
    let wanted: Vec<char> = text.chars().collect();
    let last_offset = u16::try_from(wanted.len().checked_sub(1)?).ok()?;
    let area = area.intersection(buffer.area);
    (area.top()..area.bottom()).find_map(|y| {
        let row: Vec<char> = (area.left()..area.right())
            .map(|x| cell_character(buffer, x, y))
            .collect();
        let start = row.windows(wanted.len()).position(|w| w == wanted)?;
        let x = area.left().checked_add(u16::try_from(start).ok()?)?;
        Some(Found {
            first: (x, y),
            last: (x.checked_add(last_offset)?, y),
        })
    })
}

/// How the start of some text on screen is highlighted.
pub fn highlight(buffer: &Buffer, found: Found) -> &'static str {
    match buffer.cell(found.first) {
        Some(cell) if cell.modifier.contains(Modifier::REVERSED) => "reversed",
        Some(cell) if cell.fg == Color::Red => "red",
        Some(cell) if cell.fg == Color::Yellow => "yellow",
        _ => "none",
    }
}

fn cell_character(buffer: &Buffer, x: u16, y: u16) -> char {
    buffer
        .cell((x, y))
        .and_then(|cell| cell.symbol().chars().next())
        .unwrap_or(' ')
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::text::Line;

    fn screen(lines: &[&str]) -> Buffer {
        Buffer::with_lines(lines.iter().map(|l| Line::from(*l)))
    }

    #[test]
    fn finds_the_first_and_last_character() {
        let buffer = screen(&["          ", "  mul?    "]);
        assert_eq!(
            find(&buffer, buffer.area, "mul?"),
            Some(Found {
                first: (2, 1),
                last: (5, 1)
            })
        );
    }

    #[test]
    fn searches_from_the_top() {
        let buffer = screen(&["  x", "x  "]);
        assert_eq!(find(&buffer, buffer.area, "x").unwrap().first, (2, 0));
    }

    #[test]
    fn missing_text_is_not_found() {
        let buffer = screen(&["abc"]);
        assert_eq!(find(&buffer, buffer.area, "abd"), None);
        assert_eq!(find(&buffer, buffer.area, ""), None);
    }

    #[test]
    fn counts_columns_not_bytes() {
        let buffer = screen(&["• item"]);
        assert_eq!(find(&buffer, buffer.area, "item").unwrap().first, (2, 0));
    }

    #[test]
    fn stays_inside_the_area() {
        let buffer = screen(&["top", "low"]);
        let bottom = Rect::new(0, 1, 3, 1);
        assert_eq!(find(&buffer, bottom, "top"), None);
        assert_eq!(find(&buffer, bottom, "low").unwrap().first, (0, 1));
    }
}
