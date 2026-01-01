#[derive(Clone, Copy)]
pub struct ColorScale {
    pub(super) r: f32,
    pub(super) g: f32,
    pub(super) b: f32,
    pub(super) a: f32,
}

impl Default for ColorScale {
    fn default() -> Self {
        Self {
            r: 1.,
            g: 1.,
            b: 1.,
            a: 1.,
        }
    }
}

impl ColorScale {
    pub const fn apply(&self, r: f32, g: f32, b: f32, a: f32) -> (f32, f32, f32, f32) {
        (self.r * r, self.g * g, self.b * b, self.a * a)
    }
}
