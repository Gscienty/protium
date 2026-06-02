use std::sync;

use crate::{renderer, utils};

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct DrawSourceArg {
    name: sync::Arc<str>,
    image: renderer::ImageID,
}

#[derive(Debug)]
pub(crate) struct Draw {
    destination: renderer::ImageID,
    sources: sync::Arc<[DrawSourceArg]>,
    vertices: sync::Arc<[f32]>,
    indexes: sync::Arc<[u32]>,
    blend: renderer::driver::Blend,
    destination_region: utils::Rectangle,
    shader: renderer::ShaderID,
    uniforms: sync::Arc<[u8]>,
    fill_rule: renderer::driver::FillRule,
}

#[derive(Debug)]
pub(crate) struct InternalDrawTriangles {
    destination: renderer::ImageID,
    sources: sync::Arc<[DrawSourceArg]>,

    vertices_offset: usize,
    vertices_count: usize,

    indexes_offset: usize,
    indexes_count: usize,

    blend: renderer::driver::Blend,
    destination_regions: Vec<renderer::driver::DstRegion>,
    shader_id: renderer::ShaderID,
    uniforms: sync::Arc<[u8]>,
    fill_rule: renderer::driver::FillRule,
}

impl InternalDrawTriangles {
    pub(crate) fn from(
        value: Draw,
        uniforms: sync::Arc<[u8]>,
        vertices_offset: usize,
        indexes_offset: usize,
    ) -> Self {
        Self {
            destination: value.destination,
            sources: value.sources,

            vertices_offset,
            vertices_count: value.vertices.len(),

            indexes_offset,
            indexes_count: value.indexes.len(),
            blend: value.blend,
            destination_regions: vec![renderer::driver::DstRegion::new(
                value.destination_region,
                value.indexes.len() as i32,
            )],
            shader_id: value.shader,
            uniforms,
            fill_rule: value.fill_rule,
        }
    }

    pub(crate) fn merge(&mut self, value: &Draw) {
        self.vertices_count += value.vertices.len();
        self.indexes_count += value.indexes.len();

        if matches!(self.destination_regions.last(), Some(region) if region.get_region() == value.destination_region)
        {
            if let Some(destination_region) = self.destination_regions.last_mut() {
                destination_region.add_count(value.indexes.len() as i32);
            }
        } else {
            self.destination_regions
                .push(renderer::driver::DstRegion::new(
                    value.destination_region,
                    value.indexes.len() as i32,
                ));
        }
    }

    pub(crate) fn mergeable(
        &self,
        vertices: &[f32],
        cmd: &Draw,
        uniforms: &[u8],
        vertex_floats_count: usize,
    ) -> bool {
        if self.shader_id != cmd.shader {
            return false;
        }
        if self.uniforms.as_ref().ne(uniforms) {
            return false;
        }
        if self.destination != cmd.destination {
            return false;
        }
        if self.sources.ne(&cmd.sources) {
            return false;
        }
        if self.blend != cmd.blend {
            return false;
        }

        if !matches!(cmd.fill_rule, renderer::driver::FillRule::FillAll)
            && Self::might_overlap_destination_regions(
                &vertices[self.vertices_offset..self.vertices_offset + self.vertices_count],
                &cmd.vertices,
                vertex_floats_count,
            )
        {
            return false;
        }

        true
    }

    fn might_overlap_destination_regions(
        vertices1: &[f32],
        vertices2: &[f32],
        vertex_floats_count: usize,
    ) -> bool {
        let (min_x1, min_y1, max_x1, max_y1) =
            Self::destination_region_from_vertices(vertices1, vertex_floats_count);
        let (min_x2, min_y2, max_x2, max_y2) =
            Self::destination_region_from_vertices(vertices2, vertex_floats_count);

        min_x1 < max_x2 + 1. && min_x2 < max_x1 + 1. && min_y1 < max_y2 + 1. && min_y2 < max_y1 + 1.
    }

    fn destination_region_from_vertices(
        vertices: &[f32],
        vertex_floats_count: usize,
    ) -> (f32, f32, f32, f32) {
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for index in (0..vertices.len()).step_by(vertex_floats_count) {
            let x = vertices[index];
            let y = vertices[index + 1];

            if x < min_x {
                min_x = x;
            }
            if y < min_y {
                min_y = y;
            }
            if max_x < x {
                max_x = x;
            }
            if max_y < y {
                max_y = y;
            }
        }

        (min_x, min_y, max_x, max_y)
    }

    pub(crate) fn destination_regions_is_empty(&self) -> bool {
        self.destination_regions.is_empty()
    }

    pub(crate) fn get_destination(&self) -> renderer::ImageID {
        self.destination
    }

    pub(crate) fn get_sources(&self) -> &[DrawSourceArg] {
        &self.sources
    }

    pub(crate) fn get_shader(&self) -> renderer::ShaderID {
        self.shader_id
    }

    pub(crate) fn get_destination_regions(&self) -> &[renderer::driver::DstRegion] {
        &self.destination_regions
    }

    pub(crate) fn get_blend(&self) -> renderer::driver::Blend {
        self.blend
    }

    pub(crate) fn get_fill_rule(&self) -> renderer::driver::FillRule {
        self.fill_rule
    }

    pub(crate) fn get_uniforms(&self) -> &[u8] {
        &self.uniforms
    }

    pub(crate) fn get_vertices_offset(&self) -> usize {
        self.vertices_offset
    }

    pub(crate) fn get_vertices_count(&self) -> usize {
        self.vertices_count
    }

    pub(crate) fn get_indexes_offset(&self) -> usize {
        self.indexes_offset
    }

    pub(crate) fn get_indexes_count(&self) -> usize {
        self.indexes_count
    }
}

impl Draw {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        destination: renderer::ImageID,
        sources: sync::Arc<[DrawSourceArg]>,
        vertices: sync::Arc<[f32]>,
        indexes: sync::Arc<[u32]>,
        blend: renderer::driver::Blend,
        destination_region: utils::Rectangle,
        shader: renderer::ShaderID,
        uniforms: sync::Arc<[u8]>,
        fill_rule: renderer::driver::FillRule,
    ) -> Self {
        Self {
            destination,
            sources,
            vertices,
            indexes,
            blend,
            destination_region,
            shader,
            uniforms,
            fill_rule,
        }
    }

    pub(crate) fn get_destination(&self) -> renderer::ImageID {
        self.destination
    }

    pub(crate) fn get_sources(&self) -> &[DrawSourceArg] {
        &self.sources
    }

    pub(crate) fn get_uniforms(&self) -> &[u8] {
        &self.uniforms
    }

    pub(crate) fn get_shader(&self) -> renderer::ShaderID {
        self.shader
    }

    pub(crate) fn get_vertices_count(&self) -> usize {
        self.vertices.len()
    }

    pub(crate) fn get_vertices(&self) -> &[f32] {
        &self.vertices
    }

    pub(crate) fn get_indexes(&self) -> &[u32] {
        &self.indexes
    }

    pub(crate) fn for_each_sources<F>(&self, mut cb: F)
    where
        F: FnMut(&DrawSourceArg),
    {
        for source in self.sources.iter() {
            cb(source);
        }
    }
}

impl DrawSourceArg {
    pub(crate) fn new(name: sync::Arc<str>, image: renderer::ImageID) -> Self {
        Self { name, image }
    }

    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }

    pub(crate) fn get_image(&self) -> renderer::ImageID {
        self.image
    }
}
