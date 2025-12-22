use std::{ptr, sync};

use crate::{renderer, utils};

use super::{backend, delay, drawing, material, packing};

#[derive(Clone, Copy)]
pub(crate) enum ImageType {
    Regular,
    Screen,
    Volatile,
}

#[derive(Clone)]
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
    pub(crate) fn new(
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
    pub(crate) fn extend_vertex(&mut self, vertex: Vec<AtlasDrawingVertexEntity>) -> &mut Self {
        self.vertex_list.extend(vertex);

        self
    }

    pub(crate) fn extend_index(&mut self, indexes: Vec<u32>) -> &mut Self {
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

impl<'a> AtlasDrawingSource<'a> {
    pub(crate) fn new(
        image: &'a Image,
        region: utils::Rectangle,
    ) -> Option<AtlasDrawingSource<'a>> {
        Some(Self { image, region })
    }
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

    pub(crate) fn draw(
        &mut self,
        region: utils::Rectangle,
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

        material::process_material_data(|data, material| {
            self.isolate(data, material, source.as_ref().map(|source| source.image));
            Option::<()>::None
        });

        let atlas_region =
            Self::atlas_region_width_padding(self.width, self.height, self.image_type, &self.node);
        let region = region.add(atlas_region.min);
        let backend = self.backend.as_ref()?;

        let drawing = material::process_material(|material| {
            drawing::Drawing::new(
                backend,
                region,
                utils::builtin_shader::get_nearest_shader(material.get_renderer()),
            )
        })?;
        if let Some(source) = source.as_ref() {
            drawing.set_source("source", source.image, source.region);
        }
        Some(drawing)
    }

    pub(crate) fn write_pixels(&mut self, pixels: Vec<u8>, region: utils::Rectangle) {
        material::process_material_data(|data, material| {
            if !data.in_frame() {
                data.append_delay(delay::DelayCommand::WritePixels((
                    self.clone(),
                    pixels,
                    region,
                )));
                return Option::<()>::None;
            }

            self.execute_write_pixels(data, material, pixels, region);
            Option::<()>::None
        });
    }

    fn execute_write_pixels(
        &mut self,
        material_data: &mut material::MaterialData,
        material: &material::Material,
        pixels: Vec<u8>,
        mut region: utils::Rectangle,
    ) {
        if self.get_atlas_id().is_none() {
            if pixels.is_empty() {
                return;
            }
            self.allocate(material_data, material, None, true);
        }

        let atlas_region = self.get_atlas_region();
        if region.ne(&utils::Rectangle::new(0, 0, self.width, self.height))
            || Self::padding_size(self.image_type) == 0
        {
            region = region.add(atlas_region.min);
            self.process_backend(|backend| {
                backend.write_pixels(pixels, region);
                Option::<()>::None
            });
            return;
        }

        let mut atlas_pixels = vec![0u8; (self.width + self.height * 4) as usize];
        let width = region.get_width();
        let atlas_width = atlas_region.get_width();
        for i in 0..region.get_height() {
            unsafe {
                ptr::copy_nonoverlapping(
                    pixels.as_ptr().add((4 * width * i) as usize),
                    atlas_pixels
                        .as_mut_ptr()
                        .add((4 * atlas_width * i) as usize),
                    (4 * width) as usize,
                );
            }
        }
        self.process_backend(|backend| {
            backend.write_pixels(atlas_pixels, atlas_region);
            Option::<()>::None
        });
    }

    fn isolate(
        &mut self,
        material_data: &mut material::MaterialData,
        material: &material::Material,
        source: Option<&Self>,
    ) {
        let (existed, new_image) = self.isolate_from_source_atlas(material_data, material, source);
        if !existed {
            self.allocate(material_data, material, None, false);
            return;
        }
        if let Some(new_image) = new_image {
            new_image.move_to(self);
        }
    }

    pub(crate) fn deallocate(&mut self) {
        material::process_material_data(|data, material| {
            if !data.in_frame() {
                data.append_delay(delay::DelayCommand::Deallocate(self.clone()));
                return Option::<()>::None;
            }

            self.execute_deallocate();
            Option::<()>::None
        });
    }

    pub(super) fn execute_deallocate(&mut self) {
        let Some(backend) = self.backend.take() else {
            return;
        };

        if let Some(node) = self.node.take() {
            let region = if let Ok(node) = node.lock() {
                node.get_region()
            } else {
                utils::Rectangle::default()
            };
            // clear
            backend.write_pixels(vec![], region);

            packing::TreeNode::free(&Some(node));
            return;
        }

        let Some(id) = backend.dispose() else {
            return;
        };
        material::process_material(|material| {
            material.del_atlas(id);
            Option::<()>::None
        });
    }

    fn move_to(self, destination: &mut Self) {
        destination.execute_deallocate();

        destination.width = self.width;
        destination.height = self.height;
        destination.image_type = self.image_type;
        destination.node = self.node;
        destination.backend = self.backend;
    }

    fn isolate_from_source_atlas(
        &self,
        material_data: &mut material::MaterialData,
        material: &material::Material,
        source: Option<&Self>,
    ) -> (bool, Option<Self>) {
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
        new_image.allocate(material_data, material, Some(atlas_id), false);

        let destination_region = utils::Rectangle::new(0, 0, self.width, self.height);
        let source_region = utils::Rectangle::new(0, 0, self.width, self.height);

        if let Some(drawing) = material::process_material(|material| {
            new_image.draw(
                destination_region,
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
                .set_fill_rule(renderer::FillRule::FillAll);

            drawing.directly_draw();
        }

        (true, Some(new_image))
    }

    fn get_atlas_id(&self) -> Option<renderer::ImageID> {
        self.process_backend(|backend| backend.get_atlas_id())
    }

    pub(crate) fn get_atlas_region(&self) -> utils::Rectangle {
        Self::atlas_region_width_padding(self.width, self.height, self.image_type, &self.node)
    }

    fn allocate(
        &mut self,
        material_data: &mut material::MaterialData,
        material: &material::Material,
        forbidden_atlas_id: Option<renderer::ImageID>,
        as_source: bool,
    ) {
        if matches!(self.image_type, ImageType::Screen) {
            self.backend = backend::Backend::new(
                material_data,
                material,
                self.width,
                self.height,
                false,
                true,
            );
            return;
        }

        let padding_size = Self::padding_size(self.image_type);
        let (width, height) = (self.width + padding_size, self.height + padding_size);
        let max_size = material.get_max_size();

        // volatile
        if !self.can_put_atlas(max_size) {
            if width > max_size || height > max_size {
                return;
            }
            self.backend = backend::Backend::new(
                material_data,
                material,
                self.width,
                self.height,
                as_source && matches!(self.image_type, ImageType::Regular),
                false,
            );
            return;
        }

        // regular, a part of material
        let self_image_id = self.process_backend(|backend| backend.get_atlas_id());

        let mut finded = false;
        material_data.for_each_atlas(|backend| {
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
        if finded {
            return;
        }

        // init page size
        let page_size = if as_source {
            material.get_min_source_size()
        } else {
            material.get_min_destination_size()
        };
        let mut w = page_size;
        while width > w {
            w *= 2;
            if w > max_size {
                return;
            }
        }
        let mut h = page_size;
        while height > h {
            h *= 2;
            if h > max_size {
                return;
            }
        }
        // regular, a part of material, but create new material
        let mut page = packing::Page::new(w, h, max_size);
        let node = page.alloc(width, height);

        self.backend =
            backend::Backend::new(material_data, material, width, height, as_source, false);
        self.node = node;
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
