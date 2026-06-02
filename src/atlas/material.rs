use std::{collections, sync};

use crate::renderer;

use super::{backend, delay};

pub struct MaterialData {
    atlas_set: collections::HashMap<renderer::ImageID, backend::Backend>,
    inframe: bool,
    inframe_source: collections::HashSet<renderer::ImageID>,
    delay: collections::VecDeque<delay::DelayCommand>,
}

pub struct Material {
    data: sync::Mutex<MaterialData>,
    renderer: renderer::Renderer,
    max_size: i32,
    min_source_size: i32,
    min_destination_size: i32,
}

unsafe impl Sync for Material {}
unsafe impl Send for Material {}

static MATERIAL: sync::OnceLock<Material> = sync::OnceLock::new();

pub(crate) fn init_material(
    canvas: &web_sys::HtmlCanvasElement,
    space: renderer::ColorSpace,
    max_vertex_count_limit: usize,
) {
    let Ok(renderer) = renderer::Renderer::create(canvas, space, max_vertex_count_limit) else {
        return;
    };

    MATERIAL.get_or_init(|| Material::new(renderer));
}

pub(crate) fn begin_frame() {
    process_material_data(|data, material| {
        material.begin_frame(data, material);

        Option::<()>::None
    });
}

pub(crate) fn end_frame() {
    process_material(|material| {
        material.end_frame();

        Option::<()>::None
    });
}

pub(crate) fn flush(endframe: bool) {
    process_material(|material| {
        material.flush(endframe);

        Option::<()>::None
    });
}

pub(crate) fn process_material<F, T>(cb: F) -> Option<T>
where
    F: FnOnce(&Material) -> Option<T>,
{
    let material = MATERIAL.get()?;

    cb(material)
}

pub fn process_material_data<F, T>(cb: F) -> Option<T>
where
    F: FnOnce(&mut MaterialData, &Material) -> Option<T>,
{
    let material = MATERIAL.get()?;

    let Ok(mut data) = material.data.lock() else {
        return None;
    };

    cb(&mut data, material)
}

impl Material {
    pub(crate) fn new(renderer: renderer::Renderer) -> Self {
        let max_size = renderer.get_max_texture_size();

        Self {
            data: sync::Mutex::new(MaterialData {
                atlas_set: collections::HashMap::new(),
                inframe: false,
                inframe_source: collections::HashSet::new(),
                delay: collections::VecDeque::new(),
            }),
            renderer,
            max_size,
            min_source_size: 1024,
            min_destination_size: 16,
        }
    }

    pub(crate) fn get_renderer(&self) -> &renderer::Renderer {
        &self.renderer
    }

    pub(super) fn get_max_size(&self) -> i32 {
        self.max_size
    }

    pub(super) fn get_min_source_size(&self) -> i32 {
        self.min_source_size
    }

    pub(super) fn get_min_destination_size(&self) -> i32 {
        self.min_destination_size
    }

    pub(super) fn del_atlas(&self, id: renderer::ImageID) {
        let Ok(mut data) = self.data.lock() else {
            return;
        };

        data.atlas_set.remove(&id);
    }

    pub(crate) fn begin_frame(&self, material_data: &mut MaterialData, material: &Material) {
        material_data.inframe = true;
        while let Some(delay) = material_data.delay.pop_front() {
            match delay {
                delay::DelayCommand::Draw((mut image, drawing)) => {
                    image.execute_draw(
                        material_data,
                        material,
                        drawing.sources,
                        drawing.vertices,
                        drawing.indexes,
                        drawing.blend,
                        drawing.destination_region,
                        drawing.shader_id,
                        drawing.uniforms,
                        drawing.fill_rule,
                    );
                }
                delay::DelayCommand::Deallocate(mut image) => {
                    image.execute_deallocate(material_data, material);
                }
                delay::DelayCommand::WritePixels((mut image, pixels, region)) => {
                    image.execute_write_pixels(material_data, material, pixels, region);
                }
            }
        }
    }

    pub(crate) fn end_frame(&self) {
        let Ok(mut data) = self.data.lock() else {
            return;
        };

        data.inframe = false;
        data.inframe_source.clear();
    }

    pub(crate) fn flush(&self, endframe: bool) {
        self.renderer.flush(endframe);
    }

    pub(crate) fn add_inframe_source(&self, id: renderer::ImageID) {
        let Ok(mut data) = self.data.lock() else {
            return;
        };
        data.add_inframe_source(id);
    }
}

impl MaterialData {
    pub(super) fn add_inframe_source(&mut self, id: renderer::ImageID) {
        self.inframe_source.insert(id);
    }

    pub(super) fn is_in_frame(&self, id: renderer::ImageID) -> bool {
        self.inframe_source.contains(&id)
    }

    pub(super) fn in_frame(&self) -> bool {
        self.inframe
    }

    pub(super) fn append_delay(&mut self, delay: delay::DelayCommand) {
        self.delay.push_back(delay);
    }

    pub(super) fn for_each_atlas<F>(&self, mut cb: F)
    where
        F: FnMut(&backend::Backend) -> bool,
    {
        for atlas in self.atlas_set.values() {
            if !cb(atlas) {
                break;
            }
        }
    }

    pub(super) fn add_atlas(&mut self, id: renderer::ImageID, backend: backend::Backend) {
        self.atlas_set.insert(id, backend);
    }

    pub(super) fn remove_atlas(&mut self, id: renderer::ImageID) {
        self.atlas_set.remove(&id);
    }
}
