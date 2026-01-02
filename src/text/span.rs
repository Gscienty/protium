use crate::{Image, ImageType, utils};
use std::{ptr, sync};

pub struct Span {
    font: sync::Arc<fontdue::Font>,
    layout: fontdue::layout::Layout<(f32, f32, f32, f32)>,
}

impl Span {
    pub(super) fn new(font: sync::Arc<fontdue::Font>) -> Self {
        Self {
            font,
            layout: fontdue::layout::Layout::new(fontdue::layout::CoordinateSystem::PositiveYUp),
        }
    }

    pub fn append(&mut self, text: &str, px: f32, rgba: (f32, f32, f32, f32)) {
        self.layout.append(
            &[self.font.clone()],
            &fontdue::layout::TextStyle::with_user_data(text, px, 0, rgba),
        );
    }

    pub fn get_layout(&self) -> Option<(usize, usize)> {
        let glyphs = self.layout.glyphs();
        let width = glyphs
            .iter()
            .map(|glyph| glyph.x as usize + glyph.width)
            .max()?;
        let height = glyphs
            .iter()
            .map(|glyph| glyph.y as usize + glyph.height)
            .max()?;

        Some((width, height))
    }

    pub fn draw(self) -> Option<Image> {
        let glyphs = self.layout.glyphs();
        let width = glyphs
            .iter()
            .map(|glyph| glyph.x as usize + glyph.width)
            .max()? as i32;
        let height = glyphs
            .iter()
            .map(|glyph| glyph.y as usize + glyph.height)
            .max()? as i32;

        let span_img = Image::new(width, height, ImageType::Regular);

        for glyph in self.layout.glyphs().iter() {
            let (metrics, mut data) = self
                .font
                .rasterize_indexed(glyph.key.glyph_index, glyph.key.px);

            for row in 0..metrics.height / 2 {
                unsafe {
                    ptr::swap_nonoverlapping(
                        data.as_mut_ptr().add(metrics.width * row),
                        data.as_mut_ptr()
                            .add(metrics.width * (metrics.height - row - 1)),
                        metrics.width,
                    );
                }
            }
            if let Some(block) = span_img.sub(utils::Rectangle::new(
                glyph.x as i32,
                0,
                glyph.width as i32,
                glyph.height as i32,
            )) {
                block.write_pixels(
                    data.into_iter()
                        .flat_map(|v| {
                            if v == 0 {
                                [0, 0, 0, 0]
                            } else {
                                [
                                    (255. * glyph.user_data.0) as u8,
                                    (255. * glyph.user_data.1) as u8,
                                    (255. * glyph.user_data.2) as u8,
                                    (v as f32 * glyph.user_data.3) as u8,
                                ]
                            }
                        })
                        .collect(),
                );
            }
        }

        Some(span_img)
    }
}
