use std::sync;

use crate::renderer;

#[derive(Debug)]
pub(crate) struct CreateShader {
    id: renderer::ShaderID,
    vertex_source: String,
    fragment_source: String,
    vertex_layout: renderer::driver::GLVertexLayout,
    uniforms: sync::Arc<[(String, renderer::driver::ShaderType)]>,
    texels: bool,
}

impl CreateShader {
    pub(crate) fn new(
        id: renderer::ShaderID,
        vertex_source: String,
        fragment_source: String,
        vertex_layout: renderer::driver::GLVertexLayout,
        uniforms: sync::Arc<[(String, renderer::driver::ShaderType)]>,
        texels: bool,
    ) -> Self {
        Self {
            id,
            vertex_source,
            fragment_source,
            vertex_layout,
            uniforms,
            texels,
        }
    }

    pub(crate) fn get_id(&self) -> renderer::ShaderID {
        self.id
    }

    pub(crate) fn get_vertex_source(&self) -> &str {
        &self.vertex_source
    }

    pub(crate) fn get_fragment_source(&self) -> &str {
        &self.fragment_source
    }

    pub(crate) fn get_uniforms(&self) -> &[(String, renderer::driver::ShaderType)] {
        &self.uniforms
    }

    pub(crate) fn get_texels(&self) -> bool {
        self.texels
    }
}

impl From<CreateShader> for renderer::driver::GLVertexLayout {
    fn from(value: CreateShader) -> Self {
        value.vertex_layout
    }
}
