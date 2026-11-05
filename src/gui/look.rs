//! How the GUI looks: its fonts and the colours of its two modes.
//!
//! The colours come from `spec/gui/design/images/tokens.png`.

use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, Theme, Visuals};
use std::sync::Arc;

const REGULAR: &str = "JetBrains Mono";
/// The family for headings and anything else that stands out.
pub(super) const STRONG: &str = "JetBrains Mono SemiBold";
/// The family for emphasis.
pub(super) const ITALIC: &str = "JetBrains Mono Italic";
/// The family for emphasis that stands out.
pub(super) const STRONG_ITALIC: &str = "JetBrains Mono SemiBold Italic";

/// Each face of the font, by the name it goes by.
const FACES: [(&str, &[u8]); 4] = [
    (
        REGULAR,
        include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf"),
    ),
    (
        STRONG,
        include_bytes!("../../assets/fonts/JetBrainsMono-SemiBold.ttf"),
    ),
    (
        ITALIC,
        include_bytes!("../../assets/fonts/JetBrainsMono-Italic.ttf"),
    ),
    (
        STRONG_ITALIC,
        include_bytes!("../../assets/fonts/JetBrainsMono-SemiBoldItalic.ttf"),
    ),
];

/// The colours of one mode.
#[derive(Debug, Clone, Copy)]
pub(super) struct Palette {
    pub page: Color32,
    pub panel: Color32,
    pub raised: Color32,
    pub hair: Color32,
    pub chip: Color32,
    pub text: Color32,
    pub heading: Color32,
    pub code: Color32,
    pub link: Color32,
    pub quote: Color32,
    pub line: Color32,
    pub passed: Color32,
    pub snoozed: Color32,
    pub failed: Color32,
    pub note: Color32,
    pub bar: Color32,
    pub bar_text: Color32,
    pub tint_passed: Color32,
    pub tint_failed: Color32,
    pub tint_snoozed: Color32,
    pub selection: Color32,
    pub hover: Color32,
}

const DARK: Palette = Palette {
    page: Color32::from_rgb(0x14, 0x16, 0x1A),
    panel: Color32::from_rgb(0x1A, 0x1D, 0x22),
    raised: Color32::from_rgb(0x22, 0x26, 0x2C),
    hair: Color32::from_rgb(0x2D, 0x32, 0x39),
    chip: Color32::from_rgb(0x26, 0x2A, 0x31),
    text: Color32::from_rgb(0xE4, 0xE6, 0xEA),
    heading: Color32::from_rgb(0xF4, 0xF5, 0xF7),
    code: Color32::from_rgb(0xD9, 0xB3, 0xFF),
    link: Color32::from_rgb(0x8A, 0xB4, 0xFF),
    quote: Color32::from_rgb(0xA3, 0xAB, 0xB8),
    line: Color32::from_rgb(0x7F, 0x88, 0x96),
    passed: Color32::from_rgb(0x5F, 0xD3, 0x8A),
    snoozed: Color32::from_rgb(0xEF, 0xC9, 0x5E),
    failed: Color32::from_rgb(0xFF, 0x8A, 0x80),
    note: Color32::from_rgb(0xC7, 0x97, 0x92),
    bar: Color32::from_rgb(0x8A, 0xB4, 0xFF),
    bar_text: Color32::from_rgb(0x10, 0x14, 0x1A),
    tint_passed: Color32::from_rgba_unmultiplied_const(95, 211, 138, 20),
    tint_failed: Color32::from_rgba_unmultiplied_const(255, 138, 128, 36),
    tint_snoozed: Color32::from_rgba_unmultiplied_const(239, 201, 94, 28),
    selection: Color32::from_rgba_unmultiplied_const(138, 180, 255, 41),
    hover: Color32::from_rgba_unmultiplied_const(255, 255, 255, 13),
};

const LIGHT: Palette = Palette {
    page: Color32::from_rgb(0xFF, 0xFF, 0xFF),
    panel: Color32::from_rgb(0xF6, 0xF7, 0xF9),
    raised: Color32::from_rgb(0xFF, 0xFF, 0xFF),
    hair: Color32::from_rgb(0xDF, 0xE3, 0xE8),
    chip: Color32::from_rgb(0xEE, 0xF0, 0xF3),
    text: Color32::from_rgb(0x1E, 0x22, 0x27),
    heading: Color32::from_rgb(0x11, 0x14, 0x1A),
    code: Color32::from_rgb(0x7A, 0x3E, 0xC8),
    link: Color32::from_rgb(0x0B, 0x57, 0xD0),
    quote: Color32::from_rgb(0x59, 0x62, 0x6D),
    line: Color32::from_rgb(0x7A, 0x83, 0x8E),
    passed: Color32::from_rgb(0x11, 0x63, 0x29),
    snoozed: Color32::from_rgb(0x7D, 0x4E, 0x00),
    failed: Color32::from_rgb(0xB3, 0x26, 0x1E),
    note: Color32::from_rgb(0x8C, 0x5A, 0x55),
    bar: Color32::from_rgb(0x0B, 0x57, 0xD0),
    bar_text: Color32::from_rgb(0xFF, 0xFF, 0xFF),
    tint_passed: Color32::from_rgba_unmultiplied_const(17, 99, 41, 18),
    tint_failed: Color32::from_rgba_unmultiplied_const(179, 38, 30, 23),
    tint_snoozed: Color32::from_rgba_unmultiplied_const(125, 78, 0, 23),
    selection: Color32::from_rgba_unmultiplied_const(11, 87, 208, 26),
    hover: Color32::from_rgba_unmultiplied_const(0, 0, 0, 10),
};

/// The colours of a mode.
pub(super) fn palette(mode: Theme) -> &'static Palette {
    match mode {
        Theme::Dark => &DARK,
        Theme::Light => &LIGHT,
    }
}

/// Sets up the fonts and both modes. Which mode shows is left to the system,
/// and egui falls back on dark when the system does not say.
pub(super) fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    for (name, bytes) in FACES {
        let face = Arc::new(FontData::from_static(bytes));
        fonts.font_data.insert(name.to_owned(), face);
    }
    // egui's own fonts stay behind these, for anything they do not have.
    let fallback = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        let names = fonts.families.entry(family).or_default();
        names.insert(0, REGULAR.to_owned());
    }
    for name in [STRONG, ITALIC, STRONG_ITALIC] {
        let mut names = vec![name.to_owned()];
        names.extend(fallback.iter().cloned());
        fonts.families.insert(FontFamily::Name(name.into()), names);
    }
    ctx.set_fonts(fonts);
    ctx.set_visuals_of(Theme::Dark, visuals(Visuals::dark(), &DARK));
    ctx.set_visuals_of(Theme::Light, visuals(Visuals::light(), &LIGHT));
}

fn visuals(mut visuals: Visuals, palette: &Palette) -> Visuals {
    visuals.panel_fill = palette.panel;
    visuals.window_fill = palette.raised;
    visuals.extreme_bg_color = palette.page;
    visuals.faint_bg_color = palette.chip;
    visuals.override_text_color = Some(palette.text);
    visuals.hyperlink_color = palette.link;
    visuals.selection.bg_fill = palette.selection;
    visuals.selection.stroke.color = palette.text;
    visuals.widgets.noninteractive.bg_stroke.color = palette.hair;
    visuals
}
