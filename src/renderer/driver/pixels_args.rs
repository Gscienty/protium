use crate::utils;

pub(crate) struct PixelsArgs {
    pixels: Vec<u8>,
    region: utils::Rectangle,
}

impl PixelsArgs {
    pub(crate) fn new(region: utils::Rectangle, pixels: Vec<u8>) -> Self {
        Self { pixels, region }
    }

    pub(super) const fn get_x(&self) -> i32 {
        self.region.min.x
    }

    pub(super) const fn get_y(&self) -> i32 {
        self.region.min.y
    }

    pub(super) const fn get_width(&self) -> i32 {
        self.region.get_width()
    }

    pub(super) const fn get_height(&self) -> i32 {
        self.region.get_height()
    }

    pub(super) fn get_pixels(&self) -> &[u8] {
        self.pixels.as_ref()
    }
}
