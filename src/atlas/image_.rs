use std::{
    collections::{self, HashSet},
    sync,
};

use crate::{renderer, utils};

use super::{drawing, material, packing};

pub(super) struct BackendData {
    page: packing::Page,
    image: renderer::Image,
    source: bool,
    screen: bool,
}

pub(super) type Backend = sync::Arc<sync::Mutex<BackendData>>;

#[derive(Clone, Copy)]
pub(crate) enum ImageType {
    Regular,
    Screen,
    Volatile,
}

pub(crate) struct Image {
    backend: Option<Backend>,
    node: packing::Tree<packing::PackingData>,

    width: i32,
    height: i32,
    image_type: ImageType,
}

pub(crate) struct DrawingSource<'a> {
    name: &'a str,
    image: &'a Image,
    region: utils::Rectangle,
}

#[derive(Default)]
struct IsolateSourceVariable {
    width: i32,
    height: i32,

    destination_atlas_offset: utils::Point,
    source_atlas_offset: utils::Point,
    destination_region: utils::Rectangle,
    source_region: utils::Rectangle,
}

impl drawing::DrawingVariable for IsolateSourceVariable {
    fn serialize_vertex(&self) -> (Vec<u8>, Vec<u32>) {
        let width = self.width + self.destination_atlas_offset.x;
        let height = self.height + self.destination_atlas_offset.y;

        (vec![], vec![0, 1, 2, 1, 2, 3])
    }

    fn set_source_region(
        &mut self,
        name: &str,
        atlas_region: utils::Rectangle,
        source_region: utils::Rectangle,
    ) {
    }

    fn serialize_uniform(&self) -> Vec<u8> {
        vec![]
    }

    fn set_destination_region(&mut self, region: utils::Rectangle) {}
}

impl Image {
    pub(crate) fn new(width: i32, height: i32, image_type: ImageType) -> Self {
        Self {
            width,
            height,
            image_type,

            backend: None,
            node: None,
        }
    }

    pub(crate) fn drawing<T>(
        &self,
        region: utils::Rectangle,
        shader: &renderer::Shader,
        sources: &[DrawingSource<'_>],
    ) -> Option<drawing::Drawing<T>>
    where
        T: drawing::DrawingVariable,
    {
        let atlas_region =
            Self::atlas_region_width_padding(self.width, self.height, self.image_type, &self.node);
        let region = region.add(atlas_region.min);
        let backend = self.backend.as_ref()?;

        self.isolate_source(sources);

        let drawing = drawing::Drawing::new(backend, region, shader);
        if let Some(drawing) = drawing.as_ref() {
            for source in sources.iter() {
                drawing.set_atlas_source(source.name, source.image, source.region);
            }
        }

        drawing
    }

    fn isolate_source(&self, sources: &[DrawingSource<'_>]) {}

    fn isolate_source_exist_atlas(&self, sources: &[DrawingSource<'_>]) -> (bool, Option<Self>) {
        let Some(atlas_id) = self.get_atlas_id() else {
            return (false, None);
        };
        if self.node.is_none() {
            return (true, None);
        };

        if sources
            .iter()
            .all(|source| !matches!(source.image.get_atlas_id(), Some(atlas_id)))
        {
            return (true, None);
        }

        let mut new_image = Self::new(self.width, self.height, self.image_type);
        let region =
            Self::atlas_region_width_padding(self.width, self.height, self.image_type, &self.node);

        new_image.alloc(Some(atlas_id), false);
        if let Some(drawing) = material::process_material(|material| {
            let shader = utils::builtin_shader::get_nearest_shader(material.get_renderer());

            new_image.drawing::<IsolateSourceVariable>(
                utils::Rectangle::new(0, 0, self.width, self.height),
                shader,
                &[DrawingSource {
                    name: "source",
                    image: self,
                    region: utils::Rectangle::new(0, 0, self.width, self.height),
                }],
            )
        }) {
            drawing
                .set_blend(renderer::BLEND_COPY)
                .set_fill_rule(renderer::FillRule::FillAll)
                .process_variable(|variable| {
                    // TODO
                })
                .draw();
        }

        (true, Some(new_image))
    }

    fn get_atlas_id(&self) -> Option<renderer::ImageID> {
        self.process_backend(|backend| {
            let Ok(backend) = backend.lock() else {
                return None;
            };
            Some(backend.get_image().get_id())
        })
    }

    pub(super) fn process_backend<F, T>(&self, cb: F) -> Option<T>
    where
        F: FnOnce(&Backend) -> Option<T>,
    {
        let backend = self.backend.as_ref()?;

        cb(backend)
    }

    pub(crate) fn get_atlas_region(&self) -> utils::Rectangle {
        Self::atlas_region_width_padding(self.width, self.height, self.image_type, &self.node)
    }

    fn alloc(&mut self, forbidden_atlas_id: Option<renderer::ImageID>, as_source: bool) {
        if matches!(self.image_type, ImageType::Screen) {
            material::process_material(|material| {
                let image = material
                    .get_renderer()
                    .create_image(self.width, self.height, true);
                let id = image.get_id();
                let backend = sync::Arc::new(sync::Mutex::new(BackendData {
                    page: packing::Page::new(
                        self.width,
                        self.height,
                        material.get_renderer().get_max_texture_size(),
                    ),
                    image,
                    source: false,
                    screen: true,
                }));

                material.add_atlas(id, backend.clone());
                self.backend.replace(backend);

                Option::<()>::None
            });
            return;
        }

        let padding_size = Self::padding_size(self.image_type);
        let (width, height) = (self.width + padding_size, self.height + padding_size);

        let Some(max_size) = material::process_material(|material| Some(material.get_max_size()))
        else {
            return;
        };

        // volatile
        if !self.can_put_atlas(max_size) {
            if width > max_size || height > max_size {
                return;
            }
            material::process_material(|material| {
                let image = material.get_renderer().create_image(width, height, false);
                let id = image.get_id();
                let backend = sync::Arc::new(sync::Mutex::new(BackendData {
                    page: packing::Page::new(width, height, max_size),
                    image,
                    source: as_source && matches!(self.image_type, ImageType::Regular),
                    screen: false,
                }));
                material.add_atlas(id, backend.clone());
                self.backend.replace(backend);

                Option::<()>::None
            });
            return;
        }

        // regular, a part of material
        let self_image_id = if let Some(data) = self.backend.as_ref()
            && let Ok(data) = data.lock()
        {
            Some(data.image.get_id())
        } else {
            None
        };
        let finded_material = material::process_material(|material| {
            let mut finded = false;
            material.for_each_atlas(|material_data, backend| {
                let Ok(mut data) = backend.lock() else {
                    return true;
                };
                if data.source != as_source {
                    return true;
                }
                let atlas_id = data.image.get_id();
                if material_data.is_in_frame(atlas_id) {
                    return true;
                }
                if matches!(forbidden_atlas_id, Some(forbidden_atlas_id) if forbidden_atlas_id == atlas_id) {
                    return true;
                }

                if let Some(self_image_id) = self_image_id && self_image_id == atlas_id {
                    return true;
                }

                if let Some(node) = data.alloc_node(material.get_renderer(), width, height) {
                    self.backend.replace(backend.clone());
                    self.node.replace(node);
                    finded = true;

                    return false;
                }
                true
            });
            Some(finded)
        });
        if matches!(finded_material, Some(true)) {
            return;
        }

        // init page size
        let Some((mut w, mut h)) = material::process_material(|material| {
            let size = if as_source {
                material.get_min_source_size()
            } else {
                material.get_min_destination_size()
            };
            Some((size, size))
        }) else {
            return;
        };
        while width > w {
            w *= 2;
            if w > max_size {
                return;
            }
        }
        while height > h {
            h *= 2;
            if h > max_size {
                return;
            }
        }
        // regular, a part of material, but create new material
        material::process_material(|material| {
            let mut page = packing::Page::new(w, h, max_size);
            let node = page.alloc(width, height)?;

            let image = material.get_renderer().create_image(width, height, false);
            let id = image.get_id();

            let backend = sync::Arc::new(sync::Mutex::new(BackendData {
                page,
                image,
                source: as_source,
                screen: false,
            }));

            material.add_atlas(id, backend.clone());
            self.backend.replace(backend);
            self.node.replace(node);

            Option::<()>::None
        });
    }

    fn atlas_region_width_padding(
        width: i32,
        height: i32,
        image_type: ImageType,
        node: &packing::Tree<packing::PackingData>,
    ) -> utils::Rectangle {
        if let Some(node) = node.as_ref() {
            let Ok(node) = node.lock() else {
                return utils::Rectangle::new(0, 0, 0, 0);
            };
            node.get_region()
        } else {
            let padding_size = Self::padding_size(image_type);

            utils::Rectangle::new(0, 0, width + padding_size, height + padding_size)
        }
    }

    #[inline(always)]
    const fn padding_size(image_type: ImageType) -> i32 {
        if matches!(image_type, ImageType::Regular) {
            1
        } else {
            0
        }
    }

    #[inline(always)]
    const fn can_put_atlas(&self, max_size: i32) -> bool {
        if matches!(self.image_type, ImageType::Regular) {
            let padding_size = Self::padding_size(self.image_type);
            self.width + padding_size <= max_size && self.height + padding_size <= max_size
        } else {
            false
        }
    }
}

impl BackendData {
    pub(super) fn get_image(&self) -> &renderer::Image {
        &self.image
    }

    fn alloc_node(
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
