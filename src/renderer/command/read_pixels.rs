use tokio::sync::mpsc;

use crate::{renderer, utils};

#[derive(Debug)]
pub(crate) struct ReadPixels {
    id: renderer::ImageID,
    regions: Vec<utils::Rectangle>,
    pixels_sender: mpsc::UnboundedSender<Vec<Vec<u8>>>,
}

impl ReadPixels {
    pub(crate) fn get_id(&self) -> renderer::ImageID {
        self.id
    }

    pub(crate) fn send(&self, data: Vec<Vec<u8>>) {
        let _ = self.pixels_sender.send(data);
    }

    pub(crate) fn get_regions(&self) -> &[utils::Rectangle] {
        &self.regions
    }
}
