use super::shader_type;

pub(super) struct UniformVariable {
    name: String,
    value: Vec<u8>,
    typ: shader_type::ShaderType,
}

impl UniformVariable {
    pub(super) fn new(name: &str, value: &[u8], typ: shader_type::ShaderType) -> Self {
        Self {
            name: name.to_owned(),
            value: value.to_owned(),
            typ,
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    pub(super) fn get_name(&self) -> &str {
        &self.name
    }

    pub(super) fn get_value(&self) -> &[u8] {
        &self.value
    }

    pub(super) fn get_type(&self) -> &shader_type::ShaderType {
        &self.typ
    }

    pub(super) fn valid(&self) -> bool {
        self.value.len() == self.typ.bytes_count()
    }
}
