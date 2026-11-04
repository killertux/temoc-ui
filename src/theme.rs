//! The colours of the screen. See `spec/viewer/themes.md`.

use crate::temoc::RunResult;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Block;

/// What `COLORTERM` says on a terminal that shows 24-bit colour.
const FULL_COLOUR_TERMS: [&str; 2] = ["truecolor", "24bit"];
/// The steps of the colour cube in the 256-colour palette.
const CUBE_STEPS: [u8; 6] = [0, 95, 135, 175, 215, 255];
/// The first entry of the colour cube, and of the grey ramp after it.
const CUBE_START: u8 = 16;
const GREYS_START: u8 = 232;

/// A colour, as red, green and blue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

const BLACK: Rgb = Rgb(0, 0, 0);
const WHITE: Rgb = Rgb(255, 255, 255);

impl Rgb {
    /// The colour of a terminal colour, when it is a known one. Palette entries
    /// below 16 are not: each terminal chooses those.
    #[must_use]
    pub fn of(color: Color) -> Option<Self> {
        match color {
            Color::Rgb(red, green, blue) => Some(Self(red, green, blue)),
            Color::Indexed(entry @ GREYS_START..) => {
                let grey = (entry - GREYS_START) * 10 + 8;
                Some(Self(grey, grey, grey))
            }
            Color::Indexed(entry @ CUBE_START..) => {
                let cube = entry - CUBE_START;
                let step = |index: u8| CUBE_STEPS.get(usize::from(index)).copied();
                Some(Self(step(cube / 36)?, step(cube / 6 % 6)?, step(cube % 6)?))
            }
            _ => None,
        }
    }

    /// The WCAG relative luminance.
    fn luminance(self) -> f64 {
        let linear = |channel: u8| {
            let value = f64::from(channel) / 255.0;
            if value <= 0.039_28 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(self.0) + 0.7152 * linear(self.1) + 0.0722 * linear(self.2)
    }

    /// The WCAG contrast ratio, from 1 for the same colour to 21 for black on
    /// white.
    #[must_use]
    pub fn contrast(self, other: Self) -> f64 {
        let (one, two) = (self.luminance(), other.luminance());
        (one.max(two) + 0.05) / (one.min(two) + 0.05)
    }

    /// Whether black text has more contrast on this colour than white text.
    #[must_use]
    pub fn is_light(self) -> bool {
        self.contrast(BLACK) > self.contrast(WHITE)
    }
}

/// The colours of the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub name: &'static str,
    pub page: Color,
    pub text: Color,
    pub heading: Color,
    pub code: Color,
    pub link: Color,
    pub quote: Color,
    /// Lines that are not text, such as the links in the graph.
    pub line: Color,
    pub passed: Color,
    pub snoozed: Color,
    pub failed: Color,
    pub bar: Color,
    pub bar_text: Color,
}

impl Theme {
    pub const DARK: Self = Self {
        name: "dark",
        page: Color::Rgb(0x16, 0x18, 0x1D),
        text: Color::Rgb(0xE6, 0xE8, 0xEC),
        heading: Color::Rgb(0x5F, 0xD7, 0xE6),
        code: Color::Rgb(0xD7, 0xA6, 0xFF),
        link: Color::Rgb(0x8A, 0xB4, 0xFF),
        quote: Color::Rgb(0xA8, 0xB0, 0xBD),
        line: Color::Rgb(0x8D, 0x96, 0xA5),
        passed: Color::Rgb(0x6E, 0xDB, 0x8F),
        snoozed: Color::Rgb(0xF5, 0xD7, 0x6E),
        failed: Color::Rgb(0xFF, 0x8A, 0x80),
        bar: Color::Rgb(0x87, 0xAF, 0xD7),
        bar_text: Color::Rgb(0x16, 0x18, 0x1D),
    };

    pub const LIGHT: Self = Self {
        name: "light",
        page: Color::Rgb(0xFF, 0xFF, 0xFF),
        text: Color::Rgb(0x1F, 0x23, 0x28),
        heading: Color::Rgb(0x0A, 0x6C, 0x74),
        code: Color::Rgb(0x7A, 0x3E, 0xC8),
        link: Color::Rgb(0x0B, 0x57, 0xD0),
        quote: Color::Rgb(0x57, 0x60, 0x6A),
        line: Color::Rgb(0x6E, 0x77, 0x81),
        passed: Color::Rgb(0x11, 0x63, 0x29),
        snoozed: Color::Rgb(0x7D, 0x4E, 0x00),
        failed: Color::Rgb(0xB3, 0x26, 0x1E),
        bar: Color::Rgb(0x1F, 0x3A, 0x5F),
        bar_text: Color::Rgb(0xFF, 0xFF, 0xFF),
    };

    pub const DARK_256: Self = Self {
        name: "dark-256",
        page: Color::Indexed(234),
        text: Color::Indexed(254),
        heading: Color::Indexed(80),
        code: Color::Indexed(183),
        link: Color::Indexed(111),
        quote: Color::Indexed(249),
        line: Color::Indexed(246),
        passed: Color::Indexed(78),
        snoozed: Color::Indexed(221),
        failed: Color::Indexed(210),
        bar: Color::Indexed(110),
        bar_text: Color::Indexed(234),
    };

    pub const LIGHT_256: Self = Self {
        name: "light-256",
        page: Color::Indexed(231),
        text: Color::Indexed(235),
        heading: Color::Indexed(23),
        code: Color::Indexed(91),
        link: Color::Indexed(26),
        quote: Color::Indexed(59),
        line: Color::Indexed(243),
        passed: Color::Indexed(22),
        snoozed: Color::Indexed(94),
        failed: Color::Indexed(124),
        bar: Color::Indexed(24),
        bar_text: Color::Indexed(231),
    };

    /// The theme with this name.
    #[must_use]
    pub fn named(name: &str) -> Option<Self> {
        [Self::DARK, Self::LIGHT, Self::DARK_256, Self::LIGHT_256]
            .into_iter()
            .find(|theme| theme.name == name)
    }

    /// The theme that suits a terminal, from its `COLORTERM` and its background.
    #[must_use]
    pub fn matching(color_term: Option<&str>, background: Option<Rgb>) -> Self {
        let full_colour = color_term.is_some_and(|term| FULL_COLOUR_TERMS.contains(&term));
        let light = background.is_some_and(Rgb::is_light);
        match (light, full_colour) {
            (false, true) => Self::DARK,
            (true, true) => Self::LIGHT,
            (false, false) => Self::DARK_256,
            (true, false) => Self::LIGHT_256,
        }
    }

    /// The opposite of this theme, with the same kind of colour.
    #[must_use]
    pub fn swapped(self) -> Self {
        match self.name {
            "dark" => Self::LIGHT,
            "dark-256" => Self::LIGHT_256,
            "light-256" => Self::DARK_256,
            _ => Self::DARK,
        }
    }

    /// Plain text on the page.
    #[must_use]
    pub fn plain(self) -> Style {
        Style::new().fg(self.text).bg(self.page)
    }

    /// The status bar.
    #[must_use]
    pub fn status_bar(self) -> Style {
        Style::new().fg(self.bar_text).bg(self.bar)
    }

    /// A line of the graph.
    #[must_use]
    pub fn graph_line(self) -> Style {
        Style::new().fg(self.line)
    }

    /// A line of the graph that touches the selected file.
    #[must_use]
    pub fn lit_graph_line(self) -> Style {
        Style::new().fg(self.heading).add_modifier(Modifier::BOLD)
    }

    /// The colour that stands for a result.
    #[must_use]
    pub fn result(self, result: RunResult) -> Color {
        match result {
            RunResult::Passed => self.passed,
            RunResult::Snoozed => self.snoozed,
            _ => self.failed,
        }
    }

    /// The frame of a pop-up or a panel.
    #[must_use]
    pub fn frame<'title>(self, title: impl Into<ratatui::text::Line<'title>>) -> Block<'title> {
        Block::bordered()
            .title(title)
            .title_style(Style::new().fg(self.text))
            .border_style(Style::new().fg(self.line))
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::matching(None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Theme; 4] = [Theme::DARK, Theme::LIGHT, Theme::DARK_256, Theme::LIGHT_256];

    fn contrast(one: Color, two: Color) -> f64 {
        Rgb::of(one).unwrap().contrast(Rgb::of(two).unwrap())
    }

    #[test]
    fn contrast_runs_from_one_to_twenty_one() {
        assert!((BLACK.contrast(BLACK) - 1.0).abs() < 1e-9);
        assert!((BLACK.contrast(WHITE) - 21.0).abs() < 1e-9);
        assert!((WHITE.contrast(BLACK) - 21.0).abs() < 1e-9);
    }

    #[test]
    fn every_kind_of_text_is_easy_to_read() {
        for theme in ALL {
            let on_page = [
                theme.text,
                theme.heading,
                theme.code,
                theme.link,
                theme.quote,
                theme.passed,
                theme.snoozed,
                theme.failed,
            ];
            for color in on_page {
                assert!(contrast(color, theme.page) >= 4.5, "{}", theme.name);
            }
            assert!(contrast(theme.bar_text, theme.bar) >= 4.5, "{}", theme.name);
        }
    }

    #[test]
    fn lines_and_the_status_bar_stand_out_from_the_page() {
        for theme in ALL {
            assert!(contrast(theme.line, theme.page) >= 3.0, "{}", theme.name);
            assert!(contrast(theme.bar, theme.page) >= 3.0, "{}", theme.name);
        }
    }

    #[test]
    fn palette_themes_only_use_the_fixed_entries() {
        for theme in [Theme::DARK_256, Theme::LIGHT_256] {
            let colors = [theme.page, theme.text, theme.bar, theme.failed];
            assert!(colors.into_iter().all(|color| Rgb::of(color).is_some()));
        }
    }

    #[test]
    fn palette_entries_have_their_usual_values() {
        assert_eq!(Rgb::of(Color::Indexed(16)), Some(BLACK));
        assert_eq!(Rgb::of(Color::Indexed(231)), Some(WHITE));
        assert_eq!(Rgb::of(Color::Indexed(24)), Some(Rgb(0, 95, 135)));
        assert_eq!(Rgb::of(Color::Indexed(232)), Some(Rgb(8, 8, 8)));
        assert_eq!(Rgb::of(Color::Indexed(255)), Some(Rgb(238, 238, 238)));
    }

    #[test]
    fn colours_the_terminal_chooses_are_unknown() {
        assert_eq!(Rgb::of(Color::Indexed(15)), None);
        assert_eq!(Rgb::of(Color::Red), None);
        assert_eq!(Rgb::of(Color::Reset), None);
    }

    #[test]
    fn every_theme_can_be_found_by_name() {
        for theme in ALL {
            assert_eq!(Theme::named(theme.name), Some(theme));
        }
        assert_eq!(Theme::named("blue"), None);
    }

    #[test]
    fn swapping_keeps_the_kind_of_colour_and_comes_back() {
        assert_eq!(Theme::DARK.swapped(), Theme::LIGHT);
        assert_eq!(Theme::DARK_256.swapped(), Theme::LIGHT_256);
        for theme in ALL {
            assert_eq!(theme.swapped().swapped(), theme);
        }
    }

    #[test]
    fn without_an_answer_the_theme_is_dark() {
        assert_eq!(Theme::matching(Some("truecolor"), None), Theme::DARK);
        assert_eq!(Theme::matching(None, None), Theme::DARK_256);
    }

    #[test]
    fn a_background_is_light_when_black_text_reads_better() {
        assert!(!Rgb(0x40, 0x40, 0x40).is_light());
        assert!(Rgb(0x80, 0x80, 0x80).is_light());
    }
}
