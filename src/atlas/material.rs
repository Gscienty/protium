use std::{collections, sync};

use crate::renderer;

use super::{backend, delay};

pub(super) struct MaterialData {
    atlas_set: collections::HashMap<renderer::ImageID, backend::Backend>,
    inframe: bool,
    inframe_source: collections::HashSet<renderer::ImageID>,
    delay: collections::VecDeque<delay::DelayCommand>,
}

pub(crate) struct Material {
    data: sync::Mutex<MaterialData>,
    renderer: renderer::Renderer,
    max_size: i32,
    min_source_size: i32,
    min_destination_size: i32,
}

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

pub(super) fn process_material<F, T>(cb: F) -> Option<T>
where
    F: FnOnce(&Material) -> Option<T>,
{
    let material = MATERIAL.get()?;

    cb(material)
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

    pub(super) fn get_renderer(&self) -> &renderer::Renderer {
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

    pub(super) fn add_atlas(&self, id: renderer::ImageID, backend: backend::Backend) {
        let Ok(mut data) = self.data.lock() else {
            return;
        };

        data.atlas_set.insert(id, backend);
    }

    pub(super) fn for_each_atlas<F>(&self, mut cb: F)
    where
        F: FnMut(&MaterialData, &backend::Backend) -> bool,
    {
        let Ok(data) = self.data.lock() else {
            return;
        };

        for atlas in data.atlas_set.values() {
            if !cb(&data, atlas) {
                break;
            }
        }
    }

    pub(crate) fn begin_frame(&self) {
        let Ok(mut data) = self.data.lock() else {
            return;
        };

        data.inframe = true;
        while let Some(delay) = data.delay.pop_front() {
            // TODO
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
        data.inframe_source.insert(id);
    }
}

impl MaterialData {
    pub(super) fn is_in_frame(&self, id: renderer::ImageID) -> bool {
        self.inframe_source.contains(&id)
    }
}
