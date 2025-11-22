use std::collections::{BTreeMap, btree_map::Iter};

use super::{ProgramID, shader_type};

pub(crate) struct Shader {
    program: ProgramID,
    uniform_types: BTreeMap<String, shader_type::ShaderType>,
    texels: bool,
}

impl Shader {
    pub(super) fn new<'a>(
        program: ProgramID,
        uniforms: impl Iterator<Item = &'a (&'a str, shader_type::ShaderType)>,
        texels: bool,
    ) -> Self {
        let mut uniform_types = BTreeMap::new();
        for (name, typ) in uniforms {
            uniform_types.insert((*name).to_owned(), typ.clone());
        }

        Self {
            program,
            uniform_types,
            texels,
        }
    }

    pub(super) fn iter_uniform_types(&self) -> Iter<'_, String, shader_type::ShaderType> {
        self.uniform_types.iter()
    }

    pub(super) fn get_program(&self) -> ProgramID {
        self.program
    }
}
