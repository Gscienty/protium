#[derive(Default, Debug, Clone)]
pub struct ShaderType {
    pub main: ShaderBasicType,
    pub sub: Vec<ShaderType>,
    pub length: usize,
}

#[allow(dead_code)]
#[derive(Default, Debug, Clone, Copy)]
pub enum ShaderBasicType {
    #[default]
    None,
    Float,
    Int,
    Vec2,
    Vec3,
    Vec4,
    IVec2,
    IVec3,
    IVec4,
    Mat2,
    Mat3,
    Mat4,
    Array,
}

impl ShaderType {
    pub(super) fn bytes_count(&self) -> usize {
        match self.main {
            ShaderBasicType::None => 0,
            ShaderBasicType::Int => 4,
            ShaderBasicType::Float => 4,
            ShaderBasicType::Vec2 | ShaderBasicType::IVec2 => 8,
            ShaderBasicType::Vec3 | ShaderBasicType::IVec3 => 12,
            ShaderBasicType::Vec4 | ShaderBasicType::IVec4 | ShaderBasicType::Mat2 => 16,
            ShaderBasicType::Mat3 => 36,
            ShaderBasicType::Mat4 => 64,
            ShaderBasicType::Array => self.length * self.sub[0].bytes_count(),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn into_array(self, length: usize) -> Self {
        Self {
            main: ShaderBasicType::Array,
            sub: vec![self],
            length,
        }
    }
}

impl From<ShaderBasicType> for ShaderType {
    fn from(value: ShaderBasicType) -> Self {
        Self {
            main: value,
            sub: vec![],
            length: 0,
        }
    }
}
