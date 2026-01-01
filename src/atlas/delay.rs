use crate::{renderer, utils};

use super::image;

#[derive(Debug)]
pub(super) enum DelayCommand {
    Draw(renderer::Drawing),
    Deallocate(image::Image),
    WritePixels((image::Image, Vec<u8>, utils::Rectangle)),
}
