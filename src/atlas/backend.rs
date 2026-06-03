use std::{cell, rc, sync};

use crate::{
    atlas::{image, packing},
    renderer::{self, command},
    utils,
};

#[derive(Debug)]
pub(crate) struct BackendData {
    page: Option<rc::Rc<cell::RefCell<packing::Page>>>,
    image: renderer::Image,
    source: bool,
}

#[derive(Debug, Clone)]
pub struct Backend {
    image_id: renderer::ImageID,
    pub(crate) data: rc::Rc<tokio::sync::RwLock<BackendData>>,
}

impl BackendData {
    pub(super) fn get_source(&self) -> bool {
        self.source
    }

    pub(super) fn get_atlas_id(&self) -> renderer::ImageID {
        self.image.get_id()
    }

    pub(super) fn allocate_node(
        &mut self,
        renderer: &renderer::Renderer,
        width: i32,
        height: i32,
    ) -> Option<packing::Node> {
        let page = self.page.as_ref()?;
        let node = page.borrow_mut().alloc(width, height)?;

        let (width, height) = page.borrow().size();
        self.extend(renderer, width, height);

        Some(node)
    }

    fn extend(&mut self, renderer: &renderer::Renderer, width: i32, height: i32) {
        let (image_width, image_height) = self.image.get_size();
        if image_width >= width && image_height >= height {
            return;
        }

        let new_image = renderer.create_image(width, height, self.image.is_screen());
        let (width, height) = new_image.get_internal_size();

        let vertices = utils::builtin_shader::nearest_quad_vertices_from_dst_src(
            0.,
            0.,
            width as f32,
            height as f32,
            0.,
            0.,
            width as f32,
            height as f32,
            1.,
            1.,
            1.,
            1.,
        );
        new_image.draw(
            [command::DrawSourceArg::new(
                "source".into(),
                self.image.get_id(),
            )]
            .into(),
            vertices,
            [0, 1, 2, 1, 2, 3].into(),
            renderer::BLEND_COPY,
            utils::Rectangle::new(0, 0, width, height),
            utils::builtin_shader::get_nearest_shader(renderer).get_id(),
            [].into(),
            renderer::FillRule::FillAll,
        );
        self.image.dispose();

        self.image = new_image;
    }

    pub(super) fn write_pixels(&self, pixels: sync::Arc<[u8]>, region: utils::Rectangle) {
        self.image.write_pixels(pixels, region);
    }

    pub(super) fn clear_pixels(&self, renderer: &renderer::Renderer, region: utils::Rectangle) {
        let (width, height) = self.image.get_size();

        if !region.is_in(&utils::Rectangle::new(0, 0, width, height)) {
            return;
        }

        self.image.draw(
            [].into(),
            utils::builtin_shader::clear_quad_vertices_from_dst_src(
                region.min.x as f32,
                region.min.y as f32,
                region.max.x as f32,
                region.max.y as f32,
            ),
            [0, 1, 2, 1, 2, 3].into(),
            renderer::BLEND_CLEAR,
            region,
            utils::builtin_shader::get_clear_shader(renderer).get_id(),
            [].into(),
            renderer::FillRule::FillAll,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw(
        &self,
        sources: sync::Arc<[command::DrawSourceArg]>,
        vertices: sync::Arc<[f32]>,
        indexes: sync::Arc<[u32]>,
        blend: renderer::Blend,
        destination_region: utils::Rectangle,
        shader: renderer::ShaderID,
        uniforms: sync::Arc<[u8]>,
        fill_rule: renderer::FillRule,
    ) {
        self.image.draw(
            sources,
            vertices,
            indexes,
            blend,
            destination_region,
            shader,
            uniforms,
            fill_rule,
        );
    }

    pub(super) fn dispose(&self) -> renderer::ImageID {
        self.image.dispose();

        self.image.get_id()
    }
}

impl Backend {
    pub(super) fn new(
        image: renderer::Image,
        page: Option<rc::Rc<cell::RefCell<packing::Page>>>,
        source: bool,
    ) -> Self {
        Self {
            image_id: image.get_id(),
            data: rc::Rc::new(tokio::sync::RwLock::new(BackendData {
                page,
                image,
                source,
            })),
        }
    }

    pub(super) fn get_id(&self) -> renderer::ImageID {
        self.image_id
    }

    pub(super) fn clear_pixels(&self, renderer: &renderer::Renderer, region: utils::Rectangle) {
        let data = self.data.blocking_read();
        data.clear_pixels(renderer, region);
    }

    pub(super) fn write_pixels(&self, pixels: sync::Arc<[u8]>, region: utils::Rectangle) {
        let data = self.data.blocking_read();
        data.write_pixels(pixels, region);
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw(
        &self,
        sources: sync::Arc<[image::DrawSourceArg]>,
        vertices: sync::Arc<[f32]>,
        indexes: sync::Arc<[u32]>,
        blend: renderer::Blend,
        destination_region: utils::Rectangle,
        shader: renderer::ShaderID,
        uniforms: sync::Arc<[u8]>,
        fill_rule: renderer::FillRule,
    ) {
        let data = self.data.blocking_read();

        data.draw(
            sources
                .iter()
                .filter_map(|source| {
                    let image_id = source.get_image_id()?;
                    Some(command::DrawSourceArg::new(source.get_name(), image_id))
                })
                .collect(),
            vertices,
            indexes,
            blend,
            destination_region,
            shader,
            uniforms,
            fill_rule,
        );
    }

    pub(super) fn dispose(&self) -> renderer::ImageID {
        let data = self.data.blocking_read();

        data.dispose()
    }
}
