use std::slice;

use super::{ProgramID, graphics, shader_type};

pub(crate) struct Shader {
    program: ProgramID,
    uniform_types: Vec<(String, shader_type::ShaderType)>,
    texels: bool,
}

impl Shader {
    pub(super) fn new<'a>(
        program: ProgramID,
        uniforms: impl Iterator<Item = &'a (String, shader_type::ShaderType)>,
        texels: bool,
    ) -> Self {
        Self {
            program,
            uniform_types: uniforms
                .map(|(name, typ)| (name.to_owned(), typ.clone()))
                .collect(),
            texels,
        }
    }

    pub(super) fn iter_uniform_types(&self) -> slice::Iter<(String, shader_type::ShaderType)> {
        self.uniform_types.iter()
    }

    pub(super) fn get_program(&self) -> ProgramID {
        self.program
    }

    pub(crate) fn dispose(&self, graphics: &graphics::Graphics) {
        graphics.get_handler().delete_program(self.program);
    }
}
