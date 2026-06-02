use std::sync;

use tokio::sync::mpsc;

use crate::{renderer, utils};

use super::{ImageID, command, driver};

#[derive(Debug)]
pub(crate) struct Image {
    id: ImageID,
    sender: mpsc::UnboundedSender<command::Command>,

    width: i32,
    height: i32,
    internal_width: i32,
    internal_height: i32,

    screen: bool,
}

impl Image {
    pub(super) fn new(
        id: ImageID,
        sender: mpsc::UnboundedSender<command::Command>,
        width: i32,
        height: i32,
        screen: bool,
    ) -> Self {
        Self {
            id,
            sender,
            width,
            height,
            internal_width: if screen {
                width
            } else {
                driver::Image::internal_image_size(width)
            },
            internal_height: if screen {
                height
            } else {
                driver::Image::internal_image_size(height)
            },
            screen,
        }
    }

    pub(crate) fn is_screen(&self) -> bool {
        self.screen
    }

    pub(crate) fn dispose(&self) {
        let _ = self.sender.send(command::Command::DisposeImage(self.id));
    }

    pub(crate) fn get_id(&self) -> ImageID {
        self.id
    }

    pub(crate) const fn get_internal_size(&self) -> (i32, i32) {
        (self.internal_width, self.internal_height)
    }

    pub(crate) const fn get_size(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw(
        &self,
        sources: sync::Arc<[command::DrawSourceArg]>,
        vertices: sync::Arc<[f32]>,
        indexes: sync::Arc<[u32]>,
        blend: renderer::driver::Blend,
        destination_region: utils::Rectangle,
        shader: renderer::ShaderID,
        uniforms: sync::Arc<[u8]>,
        fill_rule: renderer::driver::FillRule,
    ) {
        let _ = self.sender.send(command::Command::Draw(command::Draw::new(
            self.id,
            sources,
            vertices,
            indexes,
            blend,
            destination_region,
            shader,
            uniforms,
            fill_rule,
        )));
    }

    pub(crate) fn write_pixels(&self, pixels: sync::Arc<[u8]>, region: utils::Rectangle) {
        let _ = self
            .sender
            .send(command::Command::WritexPixels(command::WritePixels::new(
                self.id, pixels, region,
            )));
    }
}
