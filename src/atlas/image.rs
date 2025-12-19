use std::sync;

use crate::{renderer, utils};

use super::{backend, drawing, material, packing};

#[derive(Clone, Copy)]
pub(crate) enum ImageType {
    Regular,
    Screen,
    Volatile,
}

// TODO: use it
struct ImageData {
    backend: Option<backend::Backend>,
    node: packing::Tree<packing::PackingData>,

    width: i32,
    height: i32,
    image_type: ImageType,
}

pub(crate) struct Image {
    backend: Option<backend::Backend>,
    node: packing::Tree<packing::PackingData>,

    width: i32,
    height: i32,
    image_type: ImageType,
}

pub(crate) struct AtlasDrawingVertexEntity {
    destination_point: utils::Point,
    source_point: utils::Point,
    color: (f32, f32, f32, f32),
}

impl AtlasDrawingVertexEntity {
    fn new(
        destination_point: utils::Point,
        source_point: utils::Point,
        color: (f32, f32, f32, f32),
    ) -> Self {
        Self {
            destination_point,
            source_point,
            color,
        }
    }

    fn serialize(&self, destination_offset: utils::Point, source_offset: utils::Point) -> Vec<u8> {
        [
            (self.destination_point.x + destination_offset.x) as f32,
            (self.destination_point.y + destination_offset.y) as f32,
            (self.source_point.x + source_offset.x) as f32,
            (self.source_point.y + source_offset.y) as f32,
            self.color.0,
            self.color.1,
            self.color.2,
            self.color.3,
        ]
        .iter()
        .flat_map(|v| v.to_ne_bytes())
        .collect()
    }
}

#[derive(Default)]
pub(crate) struct AtlasDrawingVariable {
    vertex_list: Vec<AtlasDrawingVertexEntity>,
    indexes: Vec<u32>,
    destination_atlas_region: utils::Rectangle,
    source_atlas_region: utils::Rectangle,
    source_region: utils::Rectangle,

    destination_atlas_size: (i32, i32),
    source_atlas_size: (i32, i32),
}

impl AtlasDrawingVariable {
    fn extend_vertex(&mut self, vertex: Vec<AtlasDrawingVertexEntity>) -> &mut Self {
        self.vertex_list.extend(vertex);

        self
    }

    fn extend_index(&mut self, indexes: Vec<u32>) -> &mut Self {
        self.indexes.extend(indexes);

        self
    }
}

impl drawing::DrawingVariable for AtlasDrawingVariable {
    fn set_destination_region(&mut self, region: utils::Rectangle) {
        self.destination_atlas_region = region;
    }

    fn serialize_uniform(&self) -> Vec<u8> {
        utils::builtin_shader::get_nearest_uniform(
            self.destination_atlas_size,
            self.source_atlas_size,
            (
                self.source_atlas_region.min.x,
                self.source_atlas_region.min.y,
            ),
            (
                self.source_region.get_width(),
                self.source_region.get_height(),
            ),
        )
    }

    fn set_source_region(
        &mut self,
        name: &str,
        atlas_region: utils::Rectangle,
        source_region: utils::Rectangle,
    ) {
        self.source_atlas_region = atlas_region;
        self.source_region = source_region;
    }

    fn serialize_vertex(&self) -> (Vec<u8>, Vec<u32>) {
        let vertex = self
            .vertex_list
            .iter()
            .flat_map(|vertex| {
                vertex.serialize(
                    self.destination_atlas_region.min,
                    self.source_atlas_region.min,
                )
            })
            .collect();

        (vertex, self.indexes.to_owned())
    }

    fn set_destination_size(&mut self, size: (i32, i32)) {
        self.destination_atlas_size = size;
    }

    fn set_source_size(&mut self, size: (i32, i32)) {
        self.source_atlas_size = size;
    }
}

pub(crate) struct AtlasDrawingSource<'a> {
    image: &'a Image,
    region: utils::Rectangle,
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

    pub(super) fn process_backend<F, T>(&self, cb: F) -> Option<T>
    where
        F: FnOnce(&backend::Backend) -> Option<T>,
    {
        let backend = self.backend.as_ref()?;

        cb(backend)
    }

    pub(crate) fn drawing_atlas_builtin_nearest(
        &mut self,
        region: utils::Rectangle,
        shader: &renderer::Shader,
        source: Option<AtlasDrawingSource<'_>>,
    ) -> Option<drawing::Drawing<AtlasDrawingVariable>> {
        if let Some(atlas_id) = source
            .as_ref()
            .and_then(|source| source.image.get_atlas_id())
        {
            material::process_material(|material| {
                material.add_inframe_source(atlas_id);
                Option::<()>::None
            });
        }

        self.isolate(source.as_ref().map(|source| source.image));

        let atlas_region =
            Self::atlas_region_width_padding(self.width, self.height, self.image_type, &self.node);
        let region = region.add(atlas_region.min);
        let backend = self.backend.as_ref()?;

        let drawing = drawing::Drawing::new(backend, region, shader);
        if let Some(source) = source.as_ref() {
            if let Some(drawing) = drawing.as_ref() {
                drawing.set_source("source", source.image, source.region);
            }
        }
        drawing
    }

    fn isolate(&mut self, source: Option<&Self>) {
        let (existed, new_image) = self.isolate_from_source_atlas(source);
        if !existed {
            self.alloc(None, false);
            return;
        }
        if let Some(new_image) = new_image {
            new_image.move_to(self);
        }
    }

    fn move_to(self, destination: &mut Self) {
        // TODO: dealloc

        destination.width = self.width;
        destination.height = self.height;
        destination.image_type = self.image_type;
        destination.node = self.node;
        destination.backend = self.backend;
    }

    fn isolate_from_source_atlas(&self, source: Option<&Self>) -> (bool, Option<Self>) {
        let Some(atlas_id) = self.get_atlas_id() else {
            return (false, None);
        };

        if self.node.is_none() {
            return (true, None);
        }
        if !matches!(
            source.as_ref().and_then(|source| source.get_atlas_id()),
            Some(atlas_id)
        ) {
            return (true, None);
        }
        let mut new_image = Self::new(self.width, self.height, self.image_type);
        let region =
            Self::atlas_region_width_padding(self.width, self.height, self.image_type, &self.node);
        new_image.alloc(Some(atlas_id), false);

        let destination_region = utils::Rectangle::new(0, 0, self.width, self.height);
        let source_region = utils::Rectangle::new(0, 0, self.width, self.height);

        if let Some(drawing) = material::process_material(|material| {
            new_image.drawing_atlas_builtin_nearest(
                destination_region,
                utils::builtin_shader::get_nearest_shader(material.get_renderer()),
                Some(AtlasDrawingSource {
                    image: self,
                    region: source_region,
                }),
            )
        }) {
            drawing
                .process_variable(|mut variable| {
                    variable
                        .extend_vertex(vec![
                            AtlasDrawingVertexEntity::new(
                                utils::Point::new(0, 0),
                                utils::Point::new(0, 0),
                                (1., 1., 1., 1.),
                            ),
                            AtlasDrawingVertexEntity::new(
                                utils::Point::new(self.width, 0),
                                utils::Point::new(self.width, 0),
                                (1., 1., 1., 1.),
                            ),
                            AtlasDrawingVertexEntity::new(
                                utils::Point::new(0, self.height),
                                utils::Point::new(0, self.height),
                                (1., 1., 1., 1.),
                            ),
                            AtlasDrawingVertexEntity::new(
                                utils::Point::new(self.width, self.height),
                                utils::Point::new(self.width, self.height),
                                (1., 1., 1., 1.),
                            ),
                        ])
                        .extend_index(vec![0, 1, 2, 1, 2, 3]);
                })
                .set_blend(renderer::BLEND_COPY)
                .set_fill_rule(renderer::FillRule::FillAll)
                .draw();
        }

        (true, Some(new_image))
    }

    fn get_atlas_id(&self) -> Option<renderer::ImageID> {
        self.process_backend(|backend| backend.get_atlas_id())
    }

    pub(crate) fn get_atlas_region(&self) -> utils::Rectangle {
        Self::atlas_region_width_padding(self.width, self.height, self.image_type, &self.node)
    }

    fn alloc(&mut self, forbidden_atlas_id: Option<renderer::ImageID>, as_source: bool) {
        if matches!(self.image_type, ImageType::Screen) {
            self.backend = backend::Backend::new(self.width, self.height, false, true);
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
            self.backend = backend::Backend::new(
                self.width,
                self.height,
                as_source && matches!(self.image_type, ImageType::Regular),
                false,
            );
            return;
        }

        // regular, a part of material
        let self_image_id = self.process_backend(|backend| backend.get_atlas_id());
        let finded_material = material::process_material(|material| {
            let mut finded = false;
            material.for_each_atlas(|material_data, backend| {
                let Some(node) = backend.alloc_node(material.get_renderer(), width, height, |backend| {
                    if backend.get_source() != as_source {
                        return false;
                    }
                    let atlas_id = backend.get_image().get_id();
                    if material_data.is_in_frame(atlas_id) {
                        return false;
                    }
                    if matches!(forbidden_atlas_id, Some(forbidden_atlas_id) if forbidden_atlas_id == atlas_id) {
                        return false;
                    }

                    true
                }) else {
                    return true; // continue for_each atlas
                };

                self.backend.replace(backend.clone());
                self.node.replace(node);
                finded = true;
                false // break for_each atlas
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

            self.backend = backend::Backend::new(width, height, as_source, false);
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
