use crate::{color_scale, colorm, geom, renderer};

pub struct DrawOption {
    pub(super) colorm: colorm::ColorM,
    pub(super) geom: geom::GeoM,
    pub(super) blend: renderer::Blend,
    pub(super) color_scale: color_scale::ColorScale,
    pub(super) fill_rule: renderer::FillRule,
}

impl Default for DrawOption {
    fn default() -> Self {
        Self {
            colorm: colorm::ColorM::Identity,
            geom: Default::default(),
            blend: renderer::BLEND_DEFAULT,
            color_scale: Default::default(),
            fill_rule: Default::default(),
        }
    }
}

impl DrawOption {
    pub fn set_colorm(&mut self, colorm: colorm::ColorM) {
        self.colorm = colorm;
    }

    pub fn set_geom(&mut self, geom: geom::GeoM) {
        self.geom = geom;
    }

    pub fn set_blend(&mut self, blend: renderer::Blend) {
        self.blend = blend;
    }

    pub fn set_color_scale(&mut self, color_scale: color_scale::ColorScale) {
        self.color_scale = color_scale;
    }

    pub fn set_fill_rule(&mut self, fill_rule: renderer::FillRule) {
        self.fill_rule = fill_rule;
    }
}
