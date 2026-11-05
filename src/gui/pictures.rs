//! The pictures of the workspace, decoded once each.

use eframe::egui::{ColorImage, Context, TextureHandle, TextureOptions};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

/// The pictures seen so far. One that cannot be drawn stays known as such, so
/// that it is not read again on every frame.
#[derive(Default)]
pub(super) struct Pictures {
    seen: HashMap<PathBuf, Option<TextureHandle>>,
}

impl fmt::Debug for Pictures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let files = self.seen.keys();
        f.debug_list().entries(files).finish()
    }
}

impl Pictures {
    /// The picture in this file, if it can be drawn.
    pub fn get(&mut self, ctx: &Context, path: &Path) -> Option<&TextureHandle> {
        let seen = self.seen.entry(path.to_owned());
        seen.or_insert_with(|| decoded(ctx, path)).as_ref()
    }
}

/// Reads a picture. The page falls back on its alt text, so why it failed does
/// not matter.
fn decoded(ctx: &Context, path: &Path) -> Option<TextureHandle> {
    let bytes = std::fs::read(path).ok()?;
    let picture = image::load_from_memory(&bytes).ok()?.to_rgba8();
    let width = usize::try_from(picture.width()).ok()?;
    let height = usize::try_from(picture.height()).ok()?;
    let pixels = ColorImage::from_rgba_unmultiplied([width, height], picture.as_raw());
    let name = path.to_string_lossy();
    Some(ctx.load_texture(name, pixels, TextureOptions::LINEAR))
}
