use std::sync;

use crate::{atlas, renderer, utils};

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

    pub(crate) fn draw_source_arg(
        &self,
        name: &str,
        source_region: utils::Rectangle,
    ) -> atlas::DrawSourceArg {
        atlas::DrawSourceArg::from(&self.image, name, source_region)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw(
        &mut self,
        sources: Vec<atlas::DrawSourceArg>,
        vertices: Vec<f32>,
        indexes: sync::Arc<[u32]>,
        blend: renderer::Blend,
        destination_region: utils::Rectangle,
        shader_id: renderer::ShaderID,
        uniforms: sync::Arc<[u8]>,
        fill_rule: renderer::FillRule,
    ) {
        if let Some(cb) = self.modify_callback.as_ref() {
            cb();
        }
        self.last_blend = blend;
        self.image.draw(
            sources,
            vertices,
            indexes,
            blend,
            destination_region,
            shader_id,
            uniforms,
            fill_rule,
        );
    }

    pub(crate) fn clear(&mut self) {
        let (width, height) = self.get_size();
        self.fill(utils::Rectangle::new(0, 0, width, height), (0., 0., 0., 1.));
    }

    pub(crate) fn fill(
        &mut self,
        destination_region: utils::Rectangle,
        rgba: (f32, f32, f32, f32),
    ) {
        if let Some(cb) = self.modify_callback.as_ref() {
            cb();
        }

        let source = atlas::DrawSourceArg::from(
            get_white_image(),
            "source",
            utils::Rectangle::new(0, 0, 3, 3),
        );
        let (w, h) = self.get_size();
        let vertices = utils::builtin_shader::nearest_quad_vertices_from_src_atlas(
            1., 1., 2., 2., w as f32, 0., 0., h as f32, 0., 0., rgba.0, rgba.1, rgba.2, rgba.3,
        );
        let shader_id = atlas::process_material(|material| {
            Some(utils::builtin_shader::get_nearest_shader(material.get_renderer()).get_id())
        })
        .unwrap_or(0);

        self.image.draw(
            [source].into(),
            vertices.to_vec(),
            [0, 1, 2, 1, 2, 3].into(),
            if rgba.3 == 1. && self.last_blend == renderer::BLEND_SOURCE_OVER {
                renderer::BLEND_SOURCE_OVER
            } else {
                renderer::BLEND_COPY
            },
            destination_region,
            shader_id,
            [].into(),
            renderer::FillRule::FillAll,
        );
    }

    pub(crate) fn deallocate(&mut self) {
        self.image.deallocate();
    }

    pub(crate) fn write_pixels(&mut self, pixels: sync::Arc<[u8]>, region: utils::Rectangle) {
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
        image.write_pixels([0xff; 4 * 3 * 3].into(), utils::Rectangle::new(0, 0, 3, 3));

        image
    })
}
