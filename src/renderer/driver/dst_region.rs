use crate::utils;

#[derive(Debug)]
pub(crate) struct DstRegion {
    region: utils::Rectangle,
    count: i32,
}

impl DstRegion {
    pub(crate) fn new(region: utils::Rectangle, count: i32) -> Self {
        Self { region, count }
    }

    pub(crate) fn get_region(&self) -> &utils::Rectangle {
        &self.region
    }

    pub(crate) fn add_count(&mut self, count: i32) {
        self.count += count;
    }

    pub(super) fn get_x(&self) -> i32 {
        self.region.min.x
    }

    pub(super) fn get_y(&self) -> i32 {
        self.region.min.y
    }

    pub(super) fn get_width(&self) -> i32 {
        self.region.get_width()
    }

    pub(super) fn get_height(&self) -> i32 {
        self.region.get_height()
    }

    pub(super) fn get_count(&self) -> i32 {
        self.count
    }
}
