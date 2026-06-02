use std::{cell, collections, ptr, rc, sync};

use crate::{renderer, utils};

use super::{backend, delay, material, packing};

#[derive(Clone, Copy, Debug)]
pub enum ImageType {
    Regular,
    Screen,
    Volatile,
}

#[derive(Debug)]
pub(super) struct ImageData {
    pub(super) backend: Option<backend::Backend>,
    pub(super) node: Option<packing::Node>,
}

unsafe impl Sync for ImageData {}
unsafe impl Send for ImageData {}

#[derive(Clone, Debug)]
pub(crate) struct Image {
    data: sync::Arc<tokio::sync::RwLock<ImageData>>,

    width: i32,
    height: i32,
    image_type: ImageType,
}

#[derive(Debug, Clone)]
pub(crate) struct DrawSourceArg {
    name: sync::Arc<str>,
    width: i32,
    height: i32,
    image_type: ImageType,
    data: sync::Arc<tokio::sync::RwLock<ImageData>>,
    pub(super) source_region: utils::Rectangle,
}

impl Image {
    pub(crate) fn new(width: i32, height: i32, image_type: ImageType) -> Self {
        Self {
            data: sync::Arc::new(tokio::sync::RwLock::new(ImageData {
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

    pub(crate) fn write_pixels(&mut self, pixels: sync::Arc<[u8]>, region: utils::Rectangle) {
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
    #[allow(clippy::too_many_arguments)]
    pub(super) fn execute_write_pixels(
        &mut self,
        material_data: &mut material::MaterialData,
        material: &material::Material,
        pixels: sync::Arc<[u8]>,
        mut region: utils::Rectangle,
    ) {
        let backend_is_none = {
            let data = self.data.blocking_read();
            data.backend.is_none()
        };

        if backend_is_none {
            if pixels.is_empty() {
                return;
            }
            self.allocate(material_data, material, Default::default(), true);
        }

        let data = self.data.blocking_read();
        let r = region_with_padding(self.width, self.height, self.image_type, data.node.as_ref());

        let Some(backend) = data.backend.as_ref() else {
            return;
        };

        if region.ne(&utils::Rectangle::new(0, 0, self.width, self.height))
            || padding_size(self.image_type) == 0
        {
            region = region.add(r.min);

            if pixels.is_empty() {
                backend.clear_pixels(material.get_renderer(), region);
            } else {
                backend.write_pixels(pixels, region);
            }
            return;
        }

        let mut atlas_pixels = vec![0u8; (r.get_width() * r.get_height() * 4) as usize];
        let width = region.get_width() as usize;
        let atlas_width = r.get_width() as usize;
        for i in 0..region.get_height() as usize {
            unsafe {
                ptr::copy_nonoverlapping(
                    pixels.as_ptr().add(4 * width * i),
                    atlas_pixels.as_mut_ptr().add(4 * atlas_width * i),
                    4 * width,
                );
            }
        }
        backend.write_pixels(atlas_pixels.into(), r);
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw(
        &mut self,
        sources: Vec<DrawSourceArg>,
        vertices: Vec<f32>,
        indexes: sync::Arc<[u32]>,
        blend: renderer::Blend,
        destination_region: utils::Rectangle,
        shader_id: renderer::ShaderID,
        uniforms: sync::Arc<[u8]>,
        fill_rule: renderer::FillRule,
    ) {
        material::process_material_data(|data, material| {
            if !data.in_frame() {
                data.append_delay(delay::DelayCommand::Draw((
                    self.clone(),
                    delay::Draw {
                        sources,
                        vertices,
                        indexes,
                        blend,
                        destination_region,
                        shader_id,
                        uniforms,
                        fill_rule,
                    },
                )));
                return Option::<()>::None;
            }

            self.execute_draw(
                data,
                material,
                sources,
                vertices,
                indexes,
                blend,
                destination_region,
                shader_id,
                uniforms,
                fill_rule,
            );
            Option::<()>::None
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn execute_draw(
        &mut self,
        material_data: &mut material::MaterialData,
        material: &material::Material,
        mut sources: Vec<DrawSourceArg>,
        mut vertices: Vec<f32>,
        indexes: sync::Arc<[u32]>,
        blend: renderer::Blend,
        destination_region: utils::Rectangle,
        shader_id: renderer::ShaderID,
        uniforms: sync::Arc<[u8]>,
        fill_rule: renderer::FillRule,
    ) {
        for source in sources.iter() {
            let Some(image_id) = source.get_image_id() else {
                continue;
            };
            material_data.add_inframe_source(image_id);
        }
        self.ensure_isolate_from_source(material_data, material, &sources);

        let data = self.data.blocking_read();

        let r = region_with_padding(self.width, self.height, self.image_type, data.node.as_ref());
        let destination_region = destination_region.add(r.min);

        material.get_renderer().vertex_offset(
            shader_id,
            &mut vertices,
            renderer::LayoutVertexOffset::Destination,
            (r.min.x as f32, r.min.y as f32),
        );
        for (index, source) in sources.iter_mut().enumerate() {
            let Some(region) = source.get_region() else {
                continue;
            };
            material.get_renderer().vertex_offset(
                shader_id,
                &mut vertices,
                renderer::LayoutVertexOffset::Source(index),
                (region.min.x as f32, region.min.y as f32),
            );

            source.source_region = source.source_region.add(region.min);
        }

        let Some(backend) = data.backend.as_ref() else {
            return;
        };
        backend.draw(
            sources.into(),
            vertices.into(),
            indexes,
            blend,
            destination_region,
            shader_id,
            uniforms,
            fill_rule,
        );
    }

    fn ensure_isolate_from_source(
        &mut self,
        material_data: &mut material::MaterialData,
        material: &material::Material,
        sources: &[DrawSourceArg],
    ) {
        let (exist_backend, new_image) =
            self.ensure_isolate_from_source_exist_atlas(material_data, material, sources);
        if !exist_backend {
            let mut forbidden = collections::HashSet::new();
            material_data.for_each_atlas(|backend| {
                let id = backend.get_id();
                if material_data.is_in_frame(id) {
                    forbidden.insert(id);
                }
                true
            });

            self.allocate(material_data, material, forbidden, false);
            return;
        }

        let Some(new_image) = new_image else {
            return;
        };
        new_image.move_to(material_data, material, self);
    }

    fn move_to(
        self,
        material_data: &mut material::MaterialData,
        material: &material::Material,
        destination: &mut Self,
    ) {
        destination.execute_deallocate(material_data, material);

        destination.width = self.width;
        destination.height = self.height;
        destination.image_type = self.image_type;
        destination.data = self.data;
    }

    fn ensure_isolate_from_source_exist_atlas(
        &mut self,
        material_data: &mut material::MaterialData,
        material: &material::Material,
        sources: &[DrawSourceArg],
    ) -> (bool, Option<Self>) {
        let (mut new_image, forbidden) = {
            let data = self.data.blocking_read();
            let Some(backend) = data.backend.as_ref() else {
                return (false, None);
            };
            let atlas_id = backend.get_id();

            if data.node.is_none() {
                return (true, None);
            }
            for source in sources.iter() {
                let Some(source_image_id) = source.get_image_id() else {
                    continue;
                };
                if source_image_id != atlas_id {
                    return (true, None);
                }
            }

            let new_image = Self::new(self.width, self.height, self.image_type);

            let mut forbidden = collections::HashSet::new();
            forbidden.insert(atlas_id);
            material_data.for_each_atlas(|backend| {
                let id = backend.get_id();
                if material_data.is_in_frame(id) {
                    forbidden.insert(id);
                }
                true
            });

            (new_image, forbidden)
        };
        new_image.allocate(material_data, material, forbidden, false);

        let (w, h) = (self.width as f32, self.height as f32);

        new_image.execute_draw(
            material_data,
            material,
            sources.to_vec(),
            utils::builtin_shader::nearest_quad_vertices_from_dst_src(
                0., 0., w, h, 0., 0., w, h, 1., 1., 1., 1.,
            )
            .to_vec(),
            [0, 1, 2, 1, 2, 3].into(),
            renderer::BLEND_COPY,
            utils::Rectangle::new(0, 0, self.width, self.height),
            utils::builtin_shader::get_nearest_shader(material.get_renderer()).get_id(),
            [].into(),
            renderer::FillRule::FillAll,
        );

        (true, Some(new_image))
    }

    fn allocate(
        &mut self,
        material_data: &mut material::MaterialData,
        material: &material::Material,
        forbidden: collections::HashSet<renderer::ImageID>,
        as_source: bool,
    ) {
        let mut data = self.data.blocking_write();
        if data.backend.is_some() {
            return;
        }

        if matches!(self.image_type, ImageType::Screen) {
            let image = material
                .get_renderer()
                .create_image(self.width, self.height, true);
            let id = image.get_id();

            let backend = backend::Backend::new(image, None, false);
            material_data.add_atlas(id, backend.clone());
            data.backend.replace(backend);
            return;
        }

        let padding_size = padding_size(self.image_type);
        let (wp, hp) = (self.width + padding_size, self.height + padding_size);

        // whole backend
        if !self.can_put_atlas(material.get_max_size()) {
            if wp > material.get_max_size() || hp > material.get_max_size() {
                return;
            }

            let image = material.get_renderer().create_image(wp, hp, false);
            let id = image.get_id();

            let backend = backend::Backend::new(
                image,
                None,
                as_source && matches!(self.image_type, ImageType::Screen),
            );
            material_data.add_atlas(id, backend.clone());
            data.backend.replace(backend);
            return;
        }

        // a part of backend
        let mut finded = false;
        material_data.for_each_atlas(|backend| {
            let mut backend_data = backend.data.blocking_write();

            if backend_data.get_source() != as_source {
                return true;
            }

            if forbidden.contains(&backend_data.get_atlas_id()) {
                return true;
            }

            if let Some(node) = backend_data.allocate_node(material.get_renderer(), wp, hp) {
                data.backend.replace(backend.clone());
                data.node.replace(node);
                finded = true;
                return false;
            }
            true
        });
        if finded {
            return;
        }

        // a part of backend, but create new backend
        let (mut width, mut height) = if as_source {
            let size = material.get_min_source_size();
            (size, size)
        } else {
            let size = material.get_min_destination_size();
            (size, size)
        };
        while wp > width {
            width *= 2;
            if width > material.get_max_size() {
                return;
            }
        }
        while hp > height {
            height *= 2;
            if height > material.get_max_size() {
                return;
            }
        }

        let page = rc::Rc::new(cell::RefCell::new(packing::Page::new(
            width,
            height,
            material.get_max_size(),
        )));
        let Some(node) = page.borrow_mut().alloc(wp, hp) else {
            return;
        };
        let image = material.get_renderer().create_image(wp, hp, false);
        let id = image.get_id();

        let backend = backend::Backend::new(image, Some(page), as_source);
        material_data.add_atlas(id, backend.clone());
        data.backend.replace(backend);
        data.node.replace(node);
    }

    pub(crate) fn deallocate(&mut self) {
        material::process_material_data(|data, material| {
            if !data.in_frame() {
                data.append_delay(delay::DelayCommand::Deallocate(self.clone()));
                return Option::<()>::None;
            }

            self.execute_deallocate(data, material);
            Option::<()>::None
        });
    }

    pub(super) fn execute_deallocate(
        &mut self,
        material_data: &mut material::MaterialData,
        material: &material::Material,
    ) {
        let mut data = self.data.blocking_write();
        let Some(backend) = data.backend.take() else {
            return;
        };

        if let Some(node) = data.node.take() {
            backend.clear_pixels(material.get_renderer(), node.borrow().get_region());
            packing::free(&node);
            return;
        }

        material_data.remove_atlas(backend.dispose());
    }

    #[inline(always)]
    const fn can_put_atlas(&self, max_size: i32) -> bool {
        if matches!(self.image_type, ImageType::Regular) {
            let padding_size = padding_size(self.image_type);
            self.width + padding_size <= max_size && self.height + padding_size <= max_size
        } else {
            false
        }
    }
}
fn region_with_padding(
    width: i32,
    height: i32,
    image_type: ImageType,
    node: Option<&packing::Node>,
) -> utils::Rectangle {
    if let Some(node) = node {
        node.borrow().get_region()
    } else {
        let padding_size = padding_size(image_type);
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

impl DrawSourceArg {
    pub(crate) fn get_name(&self) -> sync::Arc<str> {
        self.name.clone()
    }

    pub(crate) fn get_image_id(&self) -> Option<renderer::ImageID> {
        let data = self.data.blocking_read();
        let backend = data.backend.as_ref()?;

        Some(backend.get_id())
    }

    pub(crate) fn get_region(&self) -> Option<utils::Rectangle> {
        let data = self.data.blocking_read();

        Some(region_with_padding(
            self.width,
            self.height,
            self.image_type,
            data.node.as_ref(),
        ))
    }

    pub(crate) fn from(image: &Image, name: &str, source_region: utils::Rectangle) -> Self {
        Self {
            name: name.into(),
            width: image.width,
            height: image.height,
            image_type: image.image_type,
            data: image.data.clone(),
            source_region,
        }
    }
}
