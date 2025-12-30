use std::sync;

use crate::{atlas, renderer, utils};

use super::{drawing, offscreen_image};

pub(crate) struct Image {
    image: atlas::Image,
    offscreen: Option<Box<offscreen_image::OffscreenImage>>,
    modify_callback: Option<Box<dyn Fn()>>,
    last_blend: renderer::Blend,
}

impl Image {
    pub(crate) fn new(width: i32, height: i32, image_type: atlas::ImageType) -> Self {
        Self {
            image: atlas::Image::new(width, height, image_type),
            offscreen: None,
            modify_callback: None,
            last_blend: renderer::BLEND_DEFAULT,
        }
    }

    pub(crate) fn clear(&mut self) {
        let (width, height) = self.image.get_size();

        self.fill(utils::Rectangle::new(0, 0, width, height), (0., 0., 0., 0.));
    }

    pub(crate) fn fill(
        &mut self,
        destination_region: utils::Rectangle,
        rgba: (f32, f32, f32, f32),
    ) {
        if let Some(cb) = self.modify_callback.as_ref() {
            cb();
        }

        let Some(drawing) = self.image.draw(destination_region, Some(get_white_image())) else {
            return;
        };
        drawing.process_variable(|mut variable| {
            variable.quad(atlas::AtlasDrawingQuatEntity::new(
                utils::Rectangle::new(
                    0,
                    0,
                    destination_region.get_width(),
                    destination_region.get_height(),
                ),
                utils::Rectangle::new_pos(1, 1, 2, 2),
                [rgba; 4],
            ));
        });
        drawing
            .set_blend(
                if rgba.3 == 1. && matches!(self.last_blend, renderer::BLEND_SOURCE_OVER) {
                    renderer::BLEND_SOURCE_OVER
                } else {
                    renderer::BLEND_COPY
                },
            )
            .set_fill_rule(renderer::FillRule::FillAll);
        drawing.draw();
    }

    pub(crate) fn deallocate(&mut self) {
        if let Some(offscreen) = self.offscreen.as_mut() {
            offscreen.deallocate();
        }
        self.image.deallocate();
    }

    pub(crate) fn draw(
        &mut self,
        region: utils::Rectangle,
        source: Option<&'_ Image>,
        blend: renderer::Blend,
        antialias: bool,
    ) -> Option<drawing::Drawing> {
        if let Some(cb) = self.modify_callback.as_ref() {
            cb();
        }
        self.last_blend = blend;

        if antialias {
            // TODO
            return None;
        }

        self.flush();
        let drawing = self.image.draw(region, source.map(|image| &image.image))?;
        drawing.set_blend(blend);
        Some(drawing::Drawing::new(drawing))
    }

    pub(crate) fn write_pixels(&mut self, pixels: Vec<u8>, region: utils::Rectangle) {
        if let Some(cb) = self.modify_callback.as_ref() {
            cb();
        }
        self.flush();
        self.image.write_pixels(pixels, region);
    }

    pub(crate) fn flush(&mut self) {
        // TODO
    }

    pub(super) fn get_image_mut(&mut self) -> &mut atlas::Image {
        &mut self.image
    }
}

static WHITE_IMAGE: sync::OnceLock<atlas::Image> = sync::OnceLock::new();

fn get_white_image() -> &'static atlas::Image {
    WHITE_IMAGE.get_or_init(|| {
        let mut image = atlas::Image::new(3, 3, atlas::ImageType::Regular);
        image.write_pixels([0xff].repeat(4 * 3 * 3), utils::Rectangle::new(0, 0, 3, 3));

        image
    })
}
