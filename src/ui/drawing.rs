use std::cell;

use crate::{atlas, renderer};

pub(crate) struct Drawing {
    drawing: atlas::Drawing<atlas::AtlasDrawingVariable>,
}

impl Drawing {
    pub(super) fn new(drawing: atlas::Drawing<atlas::AtlasDrawingVariable>) -> Self {
        Self { drawing }
    }

    pub(crate) fn process_variable<F>(&self, cb: F) -> &Self
    where
        F: FnOnce(cell::RefMut<'_, atlas::AtlasDrawingVariable>),
    {
        self.drawing.process_variable(cb);

        self
    }

    pub(crate) fn set_fill_rule(&self, fill_rule: renderer::FillRule) -> &Self {
        self.drawing.set_fill_rule(fill_rule);

        self
    }

    pub(crate) fn draw(self) {
        self.drawing.draw();
    }
}
