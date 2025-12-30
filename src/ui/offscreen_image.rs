use crate::{atlas, renderer, utils};

use super::image;

pub(super) struct OffscreenImage {
    image: Option<image::Image>,
    image_type: atlas::Image,
    region: utils::Rectangle,
    blend: renderer::Blend,
    dirty: bool,
}

impl OffscreenImage {
    pub(super) fn deallocate(&mut self) {
        if let Some(image) = self.image.as_mut() {
            image.get_image_mut().deallocate();
        }

        self.dirty = false;
    }
}
