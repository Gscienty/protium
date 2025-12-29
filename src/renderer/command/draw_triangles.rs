use crate::{renderer, utils};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DrawTriangleSourceArg {
    name: String,
    image: renderer::ImageID,
}

#[derive(Debug)]
pub(crate) struct DrawTriangles {
    destination: renderer::ImageID,
    sources: Vec<DrawTriangleSourceArg>,
    vertices: Vec<u8>,
    indexes: Vec<u32>,
    blend: renderer::driver::Blend,
    destination_region: utils::Rectangle,
    shader: renderer::ShaderID,
    uniforms: Vec<u8>,
    fill_rule: renderer::driver::FillRule,
}

#[derive(Debug)]
pub(crate) struct InternalDrawTriangles {
    destination: renderer::ImageID,
    sources: Vec<DrawTriangleSourceArg>,

    vertices_offset: usize,
    vertices_count: usize,

    indexes_offset: usize,
    indexes_count: usize,

    blend: renderer::driver::Blend,
    destination_regions: Vec<renderer::driver::DstRegion>,
    shader_id: renderer::ShaderID,
    uniforms: Vec<u8>,
    fill_rule: renderer::driver::FillRule,
}

impl InternalDrawTriangles {
    pub(crate) fn from(
        value: DrawTriangles,
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
            uniforms: value.uniforms,
            fill_rule: value.fill_rule,
        }
    }

    pub(crate) fn merge(&mut self, value: &DrawTriangles) {
        self.vertices_count += value.vertices.len();
        self.indexes_count += value.indexes.len();

        if let Some(destination_region) = self.destination_regions.last_mut()
            && destination_region
                .get_region()
                .eq(&value.destination_region)
        {
            destination_region.add_count(value.indexes.len() as i32);
        } else {
            self.destination_regions
                .push(renderer::driver::DstRegion::new(
                    value.destination_region,
                    value.indexes.len() as i32,
                ));
        }
    }

    pub(crate) fn mergeable(&self, cmd: &DrawTriangles) -> bool {
        if self.shader_id != cmd.shader {
            return false;
        }
        if self.uniforms.ne(&cmd.uniforms) {
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

        !matches!(cmd.fill_rule, renderer::driver::FillRule::FillAll)
    }

    pub(crate) fn destination_regions_is_empty(&self) -> bool {
        self.destination_regions.is_empty()
    }

    pub(crate) fn get_destination(&self) -> renderer::ImageID {
        self.destination
    }

    pub(crate) fn get_sources(&self) -> &[DrawTriangleSourceArg] {
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

impl DrawTriangles {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        destination: renderer::ImageID,
        sources: Vec<DrawTriangleSourceArg>,
        vertices: Vec<u8>,
        indexes: Vec<u32>,
        blend: renderer::driver::Blend,
        destination_region: utils::Rectangle,
        shader: renderer::ShaderID,
        uniforms: Vec<u8>,
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

    pub(crate) fn get_shader(&self) -> renderer::ShaderID {
        self.shader
    }

    pub(crate) fn get_vertices_count(&self) -> usize {
        self.vertices.len()
    }

    pub(crate) fn get_vertices(&self) -> &[u8] {
        &self.vertices
    }

    pub(crate) fn get_indexes(&self) -> &[u32] {
        &self.indexes
    }

    pub(crate) fn for_each_sources<F>(&self, mut cb: F)
    where
        F: FnMut(&DrawTriangleSourceArg),
    {
        for source in self.sources.iter() {
            cb(source);
        }
    }
}

impl DrawTriangleSourceArg {
    pub(crate) fn new(name: String, image: renderer::ImageID) -> Self {
        Self { name, image }
    }

    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }

    pub(crate) fn get_image(&self) -> renderer::ImageID {
        self.image
    }
}
