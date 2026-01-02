use std::{ptr, sync};

use crate::{renderer, utils};

use super::{backend, delay, drawing, material, packing};

#[derive(Clone, Copy, Debug)]
pub enum ImageType {
    Regular,
    Screen,
    Volatile,
}

#[derive(Debug)]
pub(super) struct ImageData {
    pub(super) backend: Option<backend::Backend>,
    pub(super) node: packing::Tree<packing::PackingData>,
}

#[derive(Clone, Debug)]
pub(crate) struct Image {
    data: sync::Arc<sync::Mutex<ImageData>>,

    width: i32,
    height: i32,
    image_type: ImageType,
}

impl Image {
    pub(crate) fn new(width: i32, height: i32, image_type: ImageType) -> Self {
        Self {
            data: sync::Arc::new(sync::Mutex::new(ImageData {
                backend: None,
                node: None,
            })),

            width,
            height,
            image_type,
        }
    }

    pub(crate) fn get_size(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    pub(super) fn process_backend<F, T>(&self, cb: F) -> Option<T>
    where
        F: FnOnce(&backend::Backend) -> Option<T>,
    {
        let Ok(data) = self.data.lock() else {
            return None;
        };
        let backend = data.backend.as_ref()?;

        cb(backend)
    }

    pub(crate) fn draw(
        &mut self,
        region: utils::Rectangle,
        source: Option<&'_ Image>,
    ) -> Option<drawing::Drawing> {
        if let Some(atlas_id) = source.as_ref().and_then(|source| source.get_atlas_id()) {
            material::process_material(|material| {
                material.add_inframe_source(atlas_id);
                Option::<()>::None
            });
        }

        material::process_material_data(|data, material| {
            self.isolate(data, material, source);
            Option::<()>::None
        });

        let region = region.add(self.get_atlas_region().min);
        let mut drawing = drawing::Drawing::new(self.clone(), region);

        if let Some(source) = source {
            drawing.set_source("source", source.clone());
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

    pub(super) fn execute_write_pixels(
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

        let mut atlas_pixels =
            vec![0u8; (atlas_region.get_width() * atlas_region.get_height() * 4) as usize];
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
        material::process_material_data(|data, _| {
            if !data.in_frame() {
                data.append_delay(delay::DelayCommand::Deallocate(self.clone()));
                return Option::<()>::None;
            }

            self.execute_deallocate();
            Option::<()>::None
        });
    }

    pub(super) fn execute_deallocate(&mut self) {
        let Ok(mut data) = self.data.lock() else {
            return;
        };

        let Some(backend) = data.backend.take() else {
            return;
        };

        if let Some(node) = data.node.take() {
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
        destination.data = self.data;
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

        {
            let Ok(data) = self.data.lock() else {
                return (false, None);
            };
            if data.node.is_none() {
                return (true, None);
            }
        }

        if source
            .as_ref()
            .and_then(|source| source.get_atlas_id())
            .ne(&Some(atlas_id))
        {
            return (true, None);
        }
        let mut new_image = Self::new(self.width, self.height, self.image_type);
        // let region =
        //     Self::atlas_region_width_padding(self.width, self.height, self.image_type, &self.node);
        new_image.allocate(material_data, material, Some(atlas_id), false);

        let destination_region = utils::Rectangle::new(0, 0, self.width, self.height);
        // let source_region = utils::Rectangle::new(0, 0, self.width, self.height);

        if let Some(mut drawing) = new_image.draw(destination_region, Some(self)) {
            drawing.quad(drawing::AtlasDrawingQuatEntity::new(
                utils::Rectangle::new(0, 0, self.width, self.height),
                utils::Rectangle::new(0, 0, self.width, self.height),
                [(1., 1., 1., 1.); 4],
            ));
            drawing.set_blend(renderer::BLEND_COPY);
            drawing.set_fill_rule(renderer::FillRule::FillAll);
            drawing.execute_draw(material_data, material);
        }

        (true, Some(new_image))
    }

    fn get_atlas_id(&self) -> Option<renderer::ImageID> {
        self.process_backend(|backend| backend.get_atlas_id())
    }

    pub(crate) fn get_atlas_region(&self) -> utils::Rectangle {
        let Ok(data) = self.data.lock() else {
            return utils::Rectangle::new(0, 0, 0, 0);
        };
        Self::atlas_region_width_padding(self.width, self.height, self.image_type, &data.node)
    }

    fn allocate(
        &mut self,
        material_data: &mut material::MaterialData,
        material: &material::Material,
        forbidden_atlas_id: Option<renderer::ImageID>,
        as_source: bool,
    ) {
        if matches!(self.image_type, ImageType::Screen) {
            let Ok(mut data) = self.data.lock() else {
                return;
            };
            data.backend = backend::Backend::new(
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
            let Ok(mut data) = self.data.lock() else {
                return;
            };
            data.backend = backend::Backend::new(
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


            let Ok(mut data) = self.data.lock() else {
                return false;
            };
            data.backend.replace(backend.clone());
            data.node.replace(node);
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
        if let Some(backend) =
            backend::Backend::new(material_data, material, w, h, as_source, false)
        {
            let Ok(mut data) = self.data.lock() else {
                return;
            };
            data.node = backend.alloc_node(material.get_renderer(), width, height, |_| true);
            data.backend = Some(backend);
        }
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
