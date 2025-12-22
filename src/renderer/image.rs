use tokio::sync::mpsc;

use crate::utils;

use super::{ImageID, command, drawing, driver, shader};

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

    pub(crate) fn get_id(&self) -> ImageID {
        self.id
    }

    pub(crate) fn dispose(&self) {
        let _ = self.sender.send(command::Command::DisposeImage(self.id));
    }

    pub(crate) const fn get_internal_size(&self) -> (i32, i32) {
        (self.internal_width, self.internal_height)
    }

    pub(crate) const fn get_size(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    pub(crate) fn drawing(
        &self,
        region: utils::Rectangle,
        shader: &shader::Shader,
    ) -> drawing::Drawing {
        drawing::Drawing::new(self.id, region, shader.get_id(), self.sender.clone())
    }

    pub(crate) fn write_pixels(&self, pixels: Vec<u8>, region: utils::Rectangle) {
        let _ = self
            .sender
            .send(command::Command::WritexPixels(command::WritePixels::new(
                self.id, pixels, region,
            )));
    }
}
