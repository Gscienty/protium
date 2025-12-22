use std::cell;

use crate::{renderer, utils};

use super::{backend, delay, image, material, packing};

pub trait DrawingVariable: Sized + Default {
    fn set_source_region(
        &mut self,
        name: &str,
        atlas_region: utils::Rectangle,
        source_region: utils::Rectangle,
    );

    fn set_destination_size(&mut self, size: (i32, i32));

    fn set_source_size(&mut self, size: (i32, i32));

    fn set_destination_region(&mut self, region: utils::Rectangle);

    fn serialize_vertex(&self) -> (Vec<u8>, Vec<u32>);

    fn serialize_uniform(&self) -> Vec<u8>;
}

pub(crate) struct Drawing<T>
where
    T: DrawingVariable,
{
    backend: backend::Backend,
    renderer_drawing: renderer::Drawing,
    variable: cell::RefCell<T>,
}

impl<T> Drawing<T>
where
    T: DrawingVariable,
{
    pub(super) fn new(
        backend: &backend::Backend,
        region: utils::Rectangle,
        shader: &renderer::Shader,
    ) -> Option<Self> {
        let renderer_drawing = backend.draw(region, shader)?;
        let variable = cell::RefCell::new(T::default());
        variable.borrow_mut().set_destination_region(region);
        variable
            .borrow_mut()
            .set_destination_size(backend.get_image_internal_size());

        Some(Self {
            backend: backend.clone(),
            renderer_drawing,
            variable,
        })
    }

    pub(crate) fn set_source(
        &self,
        name: &str,
        image: &image::Image,
        region: utils::Rectangle,
    ) -> &Self {
        let source_atlas_region = image.get_atlas_region();
        let region = region.add(source_atlas_region.min);

        let atlas_image_id = image.process_backend(|backend| backend.get_atlas_id());
        if let Some(atlas_image_id) = atlas_image_id {
            material::process_material(|material| {
                material.add_inframe_source(atlas_image_id);
                Option::<()>::None
            });
        }

        let mut variable = self.variable.borrow_mut();
        variable.set_source_region(name, source_atlas_region, region);
        if let Some(size) = image.process_backend(|backend| Some(backend.get_image_internal_size()))
        {
            variable.set_source_size(size);
        }

        self
    }

    pub(crate) fn set_blend(&self, blend: renderer::Blend) -> &Self {
        self.renderer_drawing.set_blend(blend);

        self
    }

    pub(crate) fn process_variable<F>(&self, cb: F) -> &Self
    where
        F: FnOnce(cell::RefMut<'_, T>),
    {
        cb(self.variable.borrow_mut());

        self
    }

    pub(crate) fn set_fill_rule(&self, fill_rule: renderer::FillRule) -> &Self {
        self.renderer_drawing.set_fill_rule(fill_rule);

        self
    }

    pub(crate) fn draw(self) {
        self.prepare_draw();

        material::process_material_data(|data, material| {
            if !data.in_frame() {
                data.append_delay(delay::DelayCommand::Draw(self.renderer_drawing));
                return Option::<()>::None;
            }

            self.renderer_drawing.draw();
            Option::<()>::None
        });
    }

    pub(crate) fn directly_draw(self) {
        self.prepare_draw();
        self.renderer_drawing.draw();
    }

    fn prepare_draw(&self) {
        let variable = self.variable.borrow();

        let (vertexes, indexes) = variable.serialize_vertex();
        let uniforms = variable.serialize_uniform();

        self.renderer_drawing
            .append_vertices(vertexes)
            .append_indexes(indexes)
            .set_uniforms(uniforms);
    }
}
