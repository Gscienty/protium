use std::sync;

use crate::{renderer, utils};

use super::image;

#[derive(Debug)]
pub(super) enum DelayCommand {
    Draw((image::Image, Draw)),
    Deallocate(image::Image),
    WritePixels((image::Image, sync::Arc<[u8]>, utils::Rectangle)),
}

#[derive(Debug)]
pub(super) struct Draw {
    pub(super) sources: Vec<image::DrawSourceArg>,
    pub(super) vertices: Vec<f32>,
    pub(super) indexes: sync::Arc<[u32]>,
    pub(super) blend: renderer::Blend,
    pub(super) destination_region: utils::Rectangle,
    pub(super) shader_id: renderer::ShaderID,
    pub(super) uniforms: sync::Arc<[u8]>,
    pub(super) fill_rule: renderer::FillRule,
}
