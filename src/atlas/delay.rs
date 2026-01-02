use crate::utils;

use super::{drawing, image};

#[derive(Debug)]
pub(super) enum DelayCommand {
    Draw(drawing::Drawing),
    Deallocate(image::Image),
    WritePixels((image::Image, Vec<u8>, utils::Rectangle)),
}
