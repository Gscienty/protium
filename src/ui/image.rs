use std::sync;

use crate::{atlas, renderer, utils};

use super::drawing;

pub(crate) struct Image {
    image: atlas::Image,
    modify_callback: Option<Box<dyn Fn()>>,
    last_blend: renderer::Blend,
}

unsafe impl Send for Image {}
unsafe impl Sync for Image {}

impl Image {
    pub(crate) fn new(width: i32, height: i32, image_type: atlas::ImageType) -> Self {
        Self {
            image: atlas::Image::new(width, height, image_type),
            modify_callback: None,
            last_blend: renderer::BLEND_DEFAULT,
        }
    }

    pub(crate) fn set_modify_callback(&mut self, cb: Option<Box<dyn Fn()>>) {
        self.modify_callback = cb;
    }

    pub(crate) fn get_size(&self) -> (i32, i32) {
        self.image.get_size()
    }

    pub(crate) fn draw(
        &mut self,
        region: utils::Rectangle,
        source: Option<&'_ Image>,
        blend: renderer::Blend,
    ) -> Option<drawing::Drawing> {
        self.last_blend = blend;
        self.draw_internal(region, source, blend)
    }

    pub(super) fn draw_internal(
        &mut self,
        region: utils::Rectangle,
        source: Option<&'_ Image>,
        blend: renderer::Blend,
    ) -> Option<drawing::Drawing> {
        let mut drawing = self
            .image
            .draw(region, source.map(|image| &image.image), false)?;
        drawing.set_blend(blend);
        if let Some(cb) = self.modify_callback.as_ref() {
            cb();
        }
        Some(drawing::Drawing::new(
            drawing,
            //self.modify_callback.take(),
        ))
    }

    pub(crate) fn clear(&mut self) {
        let (width, height) = self.get_size();
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

        let Some(mut drawing) = self
            .image
            .draw(destination_region, Some(get_white_image()), false)
        else {
            return;
        };
        drawing.dst_src(atlas::AtlasDrawingQuadDstSrc::new(
            (
                0.,
                0.,
                destination_region.get_width() as f32,
                destination_region.get_height() as f32,
            ),
            (1., 1., 2., 2.),
            [rgba; 4],
        ));
        drawing.set_blend(
            if rgba.3 == 1. && matches!(self.last_blend, renderer::BLEND_SOURCE_OVER) {
                renderer::BLEND_SOURCE_OVER
            } else {
                renderer::BLEND_COPY
            },
        );
        drawing.set_fill_rule(renderer::FillRule::FillAll);
        drawing.draw();
    }

    pub(crate) fn deallocate(&mut self) {
        self.image.deallocate();
    }

    pub(crate) fn write_pixels(&mut self, pixels: Vec<u8>, region: utils::Rectangle) {
        if let Some(cb) = self.modify_callback.as_ref() {
            cb();
        }
        self.image.write_pixels(pixels, region);
    }
}

static WHITE_IMAGE: sync::OnceLock<atlas::Image> = sync::OnceLock::new();

pub(crate) fn get_white_image() -> &'static atlas::Image {
    WHITE_IMAGE.get_or_init(|| {
        let mut image = atlas::Image::new(3, 3, atlas::ImageType::Regular);
        image.write_pixels([0xff].repeat(4 * 3 * 3), utils::Rectangle::new(0, 0, 3, 3));

        image
    })
}
