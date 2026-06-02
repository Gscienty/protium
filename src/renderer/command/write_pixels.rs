use std::sync;

use crate::{
    renderer::{self, driver},
    utils,
};

#[derive(Debug)]
pub(crate) struct WritePixelsArg {
    pixels: sync::Arc<[u8]>,
    region: utils::Rectangle,
}

#[derive(Debug)]
pub(crate) struct WritePixels {
    destination: renderer::ImageID,
    arg: WritePixelsArg,
}

#[derive(Debug)]
pub(crate) struct InternalWritePixels {
    destination: renderer::ImageID,
    args: Vec<WritePixelsArg>,
}

impl WritePixels {
    pub(crate) fn new(
        destination: renderer::ImageID,
        pixels: sync::Arc<[u8]>,
        region: utils::Rectangle,
    ) -> Self {
        Self {
            destination,
            arg: WritePixelsArg { pixels, region },
        }
    }

    pub(crate) fn get_destination(&self) -> renderer::ImageID {
        self.destination
    }

    pub(crate) fn get_arg(&self) -> &WritePixelsArg {
        &self.arg
    }
}

impl From<WritePixels> for WritePixelsArg {
    fn from(value: WritePixels) -> Self {
        value.arg
    }
}

impl WritePixelsArg {
    pub(crate) fn get_region(&self) -> &utils::Rectangle {
        &self.region
    }

    pub(crate) fn clear_pixels(&mut self) {
        self.pixels = [].into();
    }
}

impl InternalWritePixels {
    pub(crate) fn new(destination: renderer::ImageID, args: Vec<WritePixelsArg>) -> Self {
        Self { destination, args }
    }

    pub(crate) fn get_destination(&self) -> renderer::ImageID {
        self.destination
    }
}

impl From<WritePixelsArg> for driver::PixelsArgs {
    fn from(value: WritePixelsArg) -> Self {
        driver::PixelsArgs::new(value.region, value.pixels)
    }
}

impl From<InternalWritePixels> for Vec<WritePixelsArg> {
    fn from(value: InternalWritePixels) -> Self {
        value.args
    }
}
