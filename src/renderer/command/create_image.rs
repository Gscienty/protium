use crate::renderer;

#[derive(Debug)]
pub(crate) struct CreateImage {
    id: renderer::ImageID,
    width: i32,
    height: i32,
    is_screen: bool,
}

impl CreateImage {
    pub(crate) fn new(id: renderer::ImageID, width: i32, height: i32, is_screen: bool) -> Self {
        Self {
            id,
            width,
            height,
            is_screen,
        }
    }

    pub(crate) fn get_is_screen(&self) -> bool {
        self.is_screen
    }

    pub(crate) fn get_width(&self) -> i32 {
        self.width
    }

    pub(crate) fn get_height(&self) -> i32 {
        self.height
    }

    pub(crate) fn get_id(&self) -> renderer::ImageID {
        self.id
    }
}
