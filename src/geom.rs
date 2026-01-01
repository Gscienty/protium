#[derive(Clone, Copy, Debug)]
pub struct GeoM {
    pub(super) a: f32,
    pub(super) b: f32,
    pub(super) c: f32,
    pub(super) d: f32,
    pub(super) tx: f32,
    pub(super) ty: f32,
}

impl Default for GeoM {
    fn default() -> Self {
        Self {
            a: 1.,
            b: 0.,
            c: 0.,
            d: 1.,
            tx: 0.,
            ty: 0.,
        }
    }
}

impl GeoM {
    pub const fn apply(&self, x: f32, y: f32) -> (f32, f32) {
        (
            self.a * x + self.b * y + self.tx,
            self.c * x + self.d * y + self.ty,
        )
    }

    pub const fn concat(mut self, other: Self) -> Self {
        let a = other.a * self.a + other.b * self.c;
        let b = other.a * self.b + other.b * self.d;
        let tx = other.a * self.tx + other.b * self.ty + other.tx;
        let c = other.c * self.a + other.d * self.c;
        let d = other.c * self.b + other.d * self.d;
        let ty = other.c * self.tx + other.d * self.ty + other.ty;

        self.a = a;
        self.b = b;
        self.tx = tx;
        self.c = c;
        self.d = d;
        self.ty = ty;

        self
    }

    pub const fn scale(mut self, x: f32, y: f32) -> Self {
        let a = self.a * x;
        let b = self.b * x;
        let tx = self.tx * x;
        let c = self.c * y;
        let d = self.d * y;
        let ty = self.ty * y;

        self.a = a;
        self.b = b;
        self.tx = tx;
        self.c = c;
        self.d = d;
        self.ty = ty;

        self
    }

    pub const fn translate(mut self, tx: f32, ty: f32) -> Self {
        self.tx += tx;
        self.ty += ty;

        self
    }

    pub fn rotate(mut self, theta: f32) -> Self {
        if theta == 0. {
            return self;
        }

        let (sin, cos) = theta.sin_cos();

        let a = cos * self.a - sin * self.c;
        let b = cos * self.b - sin * self.d;
        let tx = cos * self.tx - sin * self.ty;
        let c = sin * self.a + cos * self.c;
        let d = sin * self.b + cos * self.d;
        let ty = sin * self.tx + cos * self.ty;

        self.a = a;
        self.b = b;
        self.tx = tx;
        self.c = c;
        self.d = d;
        self.ty = ty;

        self
    }

    pub fn skew(mut self, skew_x: f32, skew_y: f32) -> Self {
        let sx = skew_x.tan();
        let sy = skew_y.tan();

        let a = self.a + self.c * sx;
        let b = self.b + self.d * sx;
        let tx = self.tx + self.ty * sx;
        let c = self.c + self.a * sy;
        let d = self.d + self.b * sy;
        let ty = self.ty + self.tx * sy;

        self.a = a;
        self.b = b;
        self.tx = tx;
        self.c = c;
        self.d = d;
        self.ty = ty;

        self
    }

    pub const fn det2x2(&self) -> f32 {
        self.a * self.d - self.b * self.c
    }

    pub const fn is_invertible(&self) -> bool {
        self.det2x2() != 0.
    }

    pub const fn invert(mut self) -> Self {
        let det = self.det2x2();
        if det == 0. {
            return self;
        }

        let a = self.d / det;
        let b = -self.b / det;
        let tx = (self.b * self.ty - self.d * self.tx) / det;
        let c = -self.c / det;
        let d = self.a / det;
        let ty = (self.c * self.tx - self.a * self.ty) / det;

        self.a = a;
        self.b = b;
        self.tx = tx;
        self.c = c;
        self.d = d;
        self.ty = ty;

        self
    }
}
