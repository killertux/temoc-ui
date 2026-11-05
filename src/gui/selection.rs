//! What is selected on the page, and what has been copied. egui keeps both to
//! itself, and only says so in what it sends to the window, which a plugin gets
//! to read.

use eframe::egui::text_selection::LabelSelectionState;
use eframe::egui::{Context, FullOutput, Key, OutputCommand, Plugin, Ui};

/// What egui has said about the text of the window.
#[derive(Debug, Default)]
pub(super) struct Selection {
    selected: Option<String>,
    copied: Option<String>,
}

impl Plugin for Selection {
    fn debug_name(&self) -> &'static str {
        "Selection"
    }

    fn on_begin_pass(&mut self, ui: &mut Ui) {
        // egui only says what is selected, and never that it no longer is.
        let over = ui.input(|input| input.pointer.any_pressed() || input.key_pressed(Key::Escape));
        if over {
            self.selected = None;
        }
    }

    fn output_hook(&mut self, ctx: &Context, output: &mut FullOutput) {
        for command in &output.platform_output.commands {
            match command {
                OutputCommand::TextSelectionSettled(text) => {
                    self.selected = Some(text.clone());
                    ctx.request_repaint();
                }
                OutputCommand::CopyText(text) => {
                    self.copied = Some(text.clone());
                    ctx.request_repaint();
                }
                _ => {}
            }
        }
    }
}

/// What is selected now, and what was copied since the last time this was asked.
pub(super) fn read(ctx: &Context) -> (Option<String>, Option<String>) {
    let plugin = ctx.plugin_or_default::<Selection>();
    let mut plugin = plugin.lock();
    let selecting = ctx
        .plugin_opt::<LabelSelectionState>()
        .is_some_and(|state| state.lock().has_selection());
    let selected = plugin.selected.clone().filter(|_| selecting);
    (selected, plugin.copied.take())
}
