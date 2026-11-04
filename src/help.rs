//! The help screen.

use crate::theme::Theme;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph};

/// How wide the column of keys is.
const KEY_WIDTH: usize = 21;

/// Every key and what it does, as `spec/keys.md` lists them.
const KEYS: [(&str, &str); 21] = [
    ("j, Down", "Scroll down one line"),
    ("k, Up", "Scroll up one line"),
    ("PageDown, Space", "Scroll down one page"),
    ("PageUp", "Scroll up one page"),
    ("g, G", "Go to the top or bottom"),
    ("Left, Right", "Scroll wide tables and code sideways"),
    ("Tab, Shift-Tab", "Focus the next or previous link"),
    ("Enter", "Follow the focused link"),
    ("h, Backspace", "Go back"),
    ("l", "Go forward"),
    ("/", "Open search"),
    ("Ctrl-t", "In search, show only test files"),
    ("], [", "Focus the next or previous test table"),
    ("r", "Run the focused table"),
    ("R", "Run the whole file"),
    ("t", "Open the file tree"),
    ("Ctrl-g", "Enter the graph view"),
    ("T", "Swap between dark and light"),
    ("v", "Start selecting text"),
    ("?", "Show or hide help"),
    ("q, Ctrl-c", "Quit"),
];

/// The help screen, which scrolls when it does not fit.
#[derive(Debug, Clone, Default)]
pub struct Help {
    /// The first key on screen.
    top: usize,
    /// How many keys fit on screen, as last drawn.
    room: usize,
}

impl Help {
    fn last_top(&self) -> usize {
        KEYS.len().saturating_sub(self.room)
    }

    /// Scrolls down, stopping with the last key on screen.
    pub fn scroll_down(&mut self, rows: usize) {
        self.top = self.top.saturating_add(rows).min(self.last_top());
    }

    /// Scrolls up, stopping at the first key.
    pub fn scroll_up(&mut self, rows: usize) {
        self.top = self.top.saturating_sub(rows);
    }

    /// Draws the keys over `area`, and adapts to its size.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect, theme: Theme) {
        let block = theme.frame("Help");
        self.room = usize::from(block.inner(area).height);
        self.top = self.top.min(self.last_top());
        let lines: Vec<Line> = KEYS
            .iter()
            .skip(self.top)
            .take(self.room)
            .map(|(key, action)| {
                let bold = Style::new().add_modifier(Modifier::BOLD);
                let key = Span::styled(format!("{key:<KEY_WIDTH$}"), bold);
                Line::from(vec![key, Span::raw(*action)])
            })
            .collect();
        frame.render_widget(Clear, area);
        let lines = Paragraph::new(lines).style(theme.plain()).block(block);
        frame.render_widget(lines, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Cell;

    fn draw(help: &mut Help, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(70, height)).unwrap();
        terminal
            .draw(|frame| help.draw(frame, frame.area(), Theme::default()))
            .unwrap();
        let buffer = terminal.backend().buffer();
        buffer.content().iter().map(Cell::symbol).collect()
    }

    /// The first two cells of each row of the table of keys in the spec.
    fn keys_in_the_spec() -> Vec<(String, String)> {
        include_str!("../spec/keys.md")
            .lines()
            .take_while(|line| !line.starts_with("## "))
            .filter(|line| line.starts_with("| "))
            .skip(1)
            .filter(|line| !line.starts_with("| ---"))
            .map(|line| {
                let cells: Vec<&str> = line.split('|').map(str::trim).collect();
                (cells[1].replace('`', ""), cells[2].to_owned())
            })
            .collect()
    }

    #[test]
    fn the_keys_are_the_ones_in_the_spec() {
        let keys: Vec<(String, String)> = KEYS
            .iter()
            .map(|(key, action)| (key.to_string(), action.to_string()))
            .collect();
        assert_eq!(keys, keys_in_the_spec());
    }

    #[test]
    fn every_key_fits_its_column() {
        assert!(KEYS.iter().all(|(key, _)| key.len() < KEY_WIDTH));
    }

    #[test]
    fn every_key_shows_on_a_tall_enough_screen() {
        let mut help = Help::default();
        let text = draw(&mut help, 23);
        assert!(KEYS.iter().all(|(_, action)| text.contains(action)));
    }

    #[test]
    fn a_short_screen_scrolls_to_the_last_key_and_stops() {
        let mut help = Help::default();
        assert!(!draw(&mut help, 7).contains("Quit"));
        help.scroll_down(100);
        let text = draw(&mut help, 7);
        assert!(text.contains("Quit"));
        assert!(text.contains("Enter the graph view"));
        assert!(!text.contains("Open the file tree"));
        help.scroll_up(100);
        assert!(draw(&mut help, 7).contains("Scroll down one line"));
    }

    #[test]
    fn a_screen_that_fits_does_not_scroll() {
        let mut help = Help::default();
        draw(&mut help, 23);
        help.scroll_down(1);
        assert_eq!(help.top, 0);
    }

    #[test]
    fn a_taller_screen_takes_back_what_was_scrolled() {
        let mut help = Help::default();
        draw(&mut help, 7);
        help.scroll_down(100);
        draw(&mut help, 23);
        assert_eq!(help.top, 0);
    }

    #[test]
    fn a_tiny_screen_draws_without_trouble() {
        for height in [0, 1, 2] {
            draw(&mut Help::default(), height);
        }
    }
}
