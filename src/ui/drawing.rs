use crate::{atlas, renderer};

pub(crate) struct Drawing {
    drawing: atlas::Drawing,
    modify_callback: Option<Box<dyn Fn()>>,
}

impl Drawing {
    pub(super) fn new(drawing: atlas::Drawing, modify_callback: Option<Box<dyn Fn()>>) -> Self {
        Self {
            drawing,
            modify_callback,
        }
    }

    pub(crate) fn set_fill_rule(&mut self, fill_rule: renderer::FillRule) {
        self.drawing.set_fill_rule(fill_rule);
    }

    // pub(crate) fn quad(&mut self, quad: atlas::AtlasDrawingQuatEntity) {
    //     self.drawing.quad(quad);
    // }

    pub(crate) fn material(&mut self, material: atlas::AtlasDrawingMaterialEntity) {
        self.drawing.material(material);
    }

    pub(crate) fn draw(self) {
        if let Some(cb) = self.modify_callback.as_ref() {
            cb();
        }

        self.drawing.draw();
    }
}
