mod blend;
mod color_space;
mod dst_region;
mod fill_rule;
mod glentity;
mod glhandler;
mod glprogram;
mod graphics;
mod graphics_context;
mod image;
mod location_cache;
mod pixels_args;
mod shader;
mod shader_type;
mod uniform_variable;

pub type ProgramID = usize;
pub type ShaderID = usize;
pub type TextureID = usize;
pub type BufferID = usize;
pub type FramebufferID = usize;
pub type RenderbufferID = usize;
pub type VertexArrayID = usize;
pub type UniformLocationID = usize;

pub use color_space::ColorSpace;

pub(super) use blend::{BLEND_CLEAR, BLEND_COPY, BLEND_DEFAULT, BLEND_SOURCE_OVER, Blend};
pub(super) use dst_region::DstRegion;
pub(super) use fill_rule::FillRule;
pub(super) use glprogram::{GLVertexEntity, GLVertexLayout, GLVertexLayoutPart};
pub(super) use graphics::Graphics;
pub(crate) use image::Image;
pub(crate) use pixels_args::PixelsArgs;
pub(super) use shader::Shader;
pub(crate) use shader_type::{ShaderBasicType, ShaderType};
