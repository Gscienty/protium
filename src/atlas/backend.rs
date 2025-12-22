use std::sync;

use crate::{renderer, utils};

use super::{material, packing};

pub(super) struct BackendData {
    page: packing::Page,
    image: renderer::Image,
    source: bool,
    screen: bool,
}

#[derive(Clone)]
pub(super) struct Backend(sync::Arc<sync::Mutex<BackendData>>);

impl Backend {
    pub(super) fn new(
        material_data: &mut material::MaterialData,
        material: &material::Material,
        width: i32,
        height: i32,
        source: bool,
        screen: bool,
    ) -> Option<Self> {
        let image = material.get_renderer().create_image(width, height, screen);
        let id = image.get_id();
        let backend = Self(sync::Arc::new(sync::Mutex::new(BackendData {
            page: packing::Page::new(
                width,
                height,
                material.get_renderer().get_max_texture_size(),
            ),
            image,
            source,
            screen,
        })));

        material_data.add_atlas(id, backend.clone());

        Some(backend)
    }

    pub(super) fn write_pixels(&self, pixels: Vec<u8>, region: utils::Rectangle) {
        let Ok(data) = self.0.lock() else {
            return;
        };
        let (width, height) = data.get_image().get_size();
        if !region.is_in(&utils::Rectangle::new(0, 0, width, height)) {
            return;
        }

        material::process_material(|material| {
            if pixels.is_empty() {
                data.get_image()
                    .drawing(
                        region,
                        utils::builtin_shader::get_clear_shader(material.get_renderer()),
                    )
                    .append_vertices({
                        let min_x = region.min.x as f32;
                        let min_y = region.min.y as f32;
                        let max_x = region.max.x as f32;
                        let max_y = region.max.y as f32;

                        vec![min_x, min_y, max_x, min_y, min_x, max_y, max_x, max_y]
                            .into_iter()
                            .flat_map(|v| v.to_ne_bytes())
                            .collect()
                    })
                    .append_indexes(vec![0, 1, 2, 1, 2, 3])
                    .set_blend(renderer::BLEND_CLEAR)
                    .set_fill_rule(renderer::FillRule::FillAll)
                    .set_uniforms(utils::builtin_shader::get_clear_uniform(
                        data.get_image().get_internal_size(),
                    ))
                    .draw();
            } else {
                data.image.write_pixels(pixels, region);
            }

            Option::<()>::None
        });
    }

    pub(super) fn dispose(&self) -> Option<renderer::ImageID> {
        let Ok(data) = self.0.lock() else {
            return None;
        };
        data.get_image().dispose();

        Some(data.get_image().get_id())
    }

    pub(super) fn draw(
        &self,
        region: utils::Rectangle,
        shader: &renderer::Shader,
    ) -> Option<renderer::Drawing> {
        let Ok(data) = self.0.lock() else {
            return None;
        };

        Some(data.get_image().drawing(region, shader))
    }

    pub(super) fn get_atlas_id(&self) -> Option<renderer::ImageID> {
        let Ok(data) = self.0.lock() else {
            return None;
        };

        Some(data.get_image().get_id())
    }

    pub(super) fn get_image_internal_size(&self) -> (i32, i32) {
        let Ok(data) = self.0.lock() else {
            return (0, 0);
        };

        data.get_image().get_internal_size()
    }

    pub(super) fn alloc_node<F>(
        &self,
        renderer: &renderer::Renderer,
        width: i32,
        height: i32,
        checker: F,
    ) -> packing::Tree<packing::PackingData>
    where
        F: FnOnce(&BackendData) -> bool,
    {
        let Ok(mut data) = self.0.lock() else {
            return None;
        };
        if !checker(&data) {
            return None;
        }

        data.alloc_node(renderer, width, height)
    }
}

impl BackendData {
    pub(super) fn get_image(&self) -> &renderer::Image {
        &self.image
    }

    pub(super) fn get_source(&self) -> bool {
        self.source
    }

    pub(super) fn alloc_node(
        &mut self,
        renderer: &renderer::Renderer,
        width: i32,
        height: i32,
    ) -> packing::Tree<packing::PackingData> {
        let node = self.page.alloc(width, height)?;

        let (page_width, page_height) = self.page.size();
        let (origin_image_width, origin_image_height) = self.image.get_internal_size();
        if page_width <= origin_image_width && page_height <= origin_image_height {
            return Some(node);
        }

        let new_image = renderer.create_image(page_width, page_height, self.screen);

        // clear
        new_image
            .drawing(
                utils::Rectangle::new(0, 0, page_width, page_height),
                utils::builtin_shader::get_clear_shader(renderer),
            )
            .append_vertices({
                let width = page_width as f32;
                let height = page_height as f32;
                vec![0., 0., width, 0., 0., height, width, height]
                    .into_iter()
                    .flat_map(|v| v.to_ne_bytes())
                    .collect()
            })
            .append_indexes(vec![0, 1, 2, 1, 2, 3])
            .set_blend(renderer::BLEND_CLEAR)
            .set_fill_rule(renderer::FillRule::FillAll)
            .set_uniforms(utils::builtin_shader::get_clear_uniform(
                new_image.get_internal_size(),
            ))
            .draw();

        new_image
            .drawing(
                utils::Rectangle::new(0, 0, origin_image_width, origin_image_height),
                utils::builtin_shader::get_nearest_shader(renderer),
            )
            .set_source("source", &self.image)
            .append_vertices({
                let width = origin_image_width as f32;
                let height = origin_image_height as f32;
                vec![
                    0., 0., 0., 0., 1., 1., 1., 1., width, 0., width, 0., 1., 1., 1., 1., 0.,
                    height, 0., height, 1., 1., 1., 1., width, height, width, height, 1., 1., 1.,
                    1.,
                ]
                .into_iter()
                .flat_map(|v| v.to_ne_bytes())
                .collect()
            })
            .append_indexes(vec![0, 1, 2, 1, 2, 3])
            .set_blend(renderer::BLEND_CLEAR)
            .set_fill_rule(renderer::FillRule::FillAll)
            .set_uniforms(utils::builtin_shader::get_nearest_uniform(
                new_image.get_internal_size(),
                self.image.get_internal_size(),
                (0, 0),
                (origin_image_width, origin_image_height),
            ))
            .draw();

        self.image.dispose();
        self.image = new_image;

        Some(node)
    }
}
