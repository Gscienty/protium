use std::cell;

use crate::{atlas, renderer};

pub(crate) enum Drawing {
    Image(Box<DrawingImage>),
}

impl Drawing {
    pub(super) fn new(
        drawing: atlas::Drawing<atlas::AtlasDrawingVariable>,
        modify_callback: Option<Box<dyn Fn()>>,
    ) -> Self {
        Self::Image(Box::new(DrawingImage::new(drawing, modify_callback)))
    }

    pub(crate) fn process_variable<F>(&self, cb: F) -> &Self
    where
        F: FnOnce(cell::RefMut<'_, atlas::AtlasDrawingVariable>),
    {
        match self {
            Self::Image(image) => {
                image.process_variable(cb);
            }
        }

        self
    }

    pub(crate) fn set_fill_rule(&mut self, fill_rule: renderer::FillRule) -> &Self {
        match self {
            Self::Image(image) => image.set_fill_rule(fill_rule),
        }

        self
    }

    pub(crate) fn draw(self) {
        match self {
            Self::Image(image) => image.draw(),
        }
    }
}

pub(crate) struct DrawingImage {
    drawing: atlas::Drawing<atlas::AtlasDrawingVariable>,
    modify_callback: Option<Box<dyn Fn()>>,
}

impl DrawingImage {
    pub(super) fn new(
        drawing: atlas::Drawing<atlas::AtlasDrawingVariable>,
        modify_callback: Option<Box<dyn Fn()>>,
    ) -> Self {
        Self {
            drawing,
            modify_callback,
        }
    }

    pub(crate) fn process_variable<F>(&self, cb: F)
    where
        F: FnOnce(cell::RefMut<'_, atlas::AtlasDrawingVariable>),
    {
        self.drawing.process_variable(cb);
    }

    pub(crate) fn set_fill_rule(&self, fill_rule: renderer::FillRule) {
        self.drawing.set_fill_rule(fill_rule);
    }

    pub(crate) fn draw(self) {
        if let Some(cb) = self.modify_callback.as_ref() {
            cb();
        }

        self.drawing.draw();
    }
}
