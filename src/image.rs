use std::sync;

use crate::{atlas, draw_option, ui, utils};

#[derive(Clone)]
pub struct Image {
    origin: Option<Box<Image>>,
    image: sync::Arc<sync::Mutex<Option<ui::Image>>>,
    bound: utils::Rectangle,
}

impl Image {
    pub fn new(width: i32, height: i32, image_type: atlas::ImageType) -> Self {
        Self {
            origin: None,
            image: sync::Arc::new(sync::Mutex::new(Some(ui::Image::new(
                width, height, image_type,
            )))),
            bound: utils::Rectangle::new(0, 0, width, height),
        }
    }

    pub fn draw(&self, source: &Self, option: draw_option::DrawOption) {
        let blend = option.blend;
        let mut geom = option.geom;
        let (offset_x, offset_y) = self.adjust_position(0, 0);
        if offset_x != 0 || offset_y != 0 {
            geom = geom.translate(offset_x as f32, offset_y as f32);
        }
        if geom.a * geom.d - geom.b * geom.c == 0. {
            return;
        }

        let bounds = source.bound;
        let (sx0, sy0) = source.adjust_position(bounds.min.x, bounds.min.y);
        let (sx1, sy1) = source.adjust_position(bounds.max.x, bounds.max.y);
        let (_, cr, cg, cb, ca) = option.colorm.to_scale();
        let (cr, cg, cb, ca) = option.color_scale.apply(cr, cb, cg, ca);

        let Ok(source) = source.image.lock() else {
            return;
        };

        let Ok(mut target) = self.image.lock() else {
            return;
        };
        let Some(target) = target.as_mut() else {
            return;
        };

        let Some(mut drawing) = target.draw(self.adjust_bounds(), source.as_ref(), blend) else {
            return;
        };
        drawing.set_fill_rule(option.fill_rule);
        drawing.material(atlas::AtlasDrawingMaterialEntity::new(
            utils::Rectangle::new_pos(sx0, sy0, sx1, sy1),
            (geom.a, geom.b, geom.c, geom.d),
            (geom.tx, geom.ty),
            (cr, cg, cb, ca),
        ));
        drawing.draw();
    }

    pub fn clear(&self) {
        self.fill((0., 0., 0., 0.));
    }

    pub fn fill(&self, rgba: (f32, f32, f32, f32)) {
        let Ok(mut target) = self.image.lock() else {
            return;
        };
        let Some(target) = target.as_mut() else {
            return;
        };
        target.fill(self.adjust_bounds(), rgba);
    }

    pub fn sub(&self, bound: utils::Rectangle) -> Option<Self> {
        let Ok(image) = self.image.lock() else {
            return None;
        };
        if image.is_none() {
            return None;
        }

        let bound = bound.intersect(self.bounds());
        Some(Self {
            image: self.image.clone(),
            origin: if self.is_sub_image() {
                self.origin.clone()
            } else {
                Some(Box::new(self.clone()))
            },
            bound: if bound.is_empty() {
                utils::Rectangle::new(0, 0, 0, 0)
            } else {
                bound
            },
        })
    }

    pub fn deallocate(&self) {
        if self.is_sub_image() {
            return;
        }
        let Ok(mut image) = self.image.lock() else {
            return;
        };
        if let Some(mut image) = image.take() {
            image.deallocate();
        }
    }

    pub fn write_pixels(&self, pixels: Vec<u8>) {
        let region = self.adjust_bounds();
        let Ok(mut image) = self.image.lock() else {
            return;
        };
        if let Some(image) = image.as_mut() {
            image.write_pixels(pixels, region);
        }
    }

    pub(super) fn get_ui_image(&self) -> sync::Arc<sync::Mutex<Option<ui::Image>>> {
        self.image.clone()
    }

    const fn adjust_position(&self, mut x: i32, mut y: i32) -> (i32, i32) {
        let region = if let Some(origin) = self.origin.as_ref() {
            origin.bound
        } else {
            self.bound
        };

        x -= region.min.x;
        y -= region.min.y;

        (x, y)
    }

    const fn adjust_bounds(&self) -> utils::Rectangle {
        let b = self.bound;
        let (x, y) = self.adjust_position(b.min.x, b.min.y);

        utils::Rectangle::new_pos(x, y, x + b.get_width(), y + b.get_height())
    }

    pub const fn bounds(&self) -> utils::Rectangle {
        self.bound
    }

    fn is_sub_image(&self) -> bool {
        self.origin.is_some()
    }
}
