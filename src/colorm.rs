#[derive(Clone, Copy)]
pub enum ColorM {
    Identity,
    Scale([f32; 4]),
    Translate([f32; 16], [f32; 4]),
}

impl ColorM {
    pub fn is_identity(&self) -> bool {
        match self {
            Self::Identity => true,
            Self::Scale(v) => v.eq(&[1., 1., 1., 1.]),
            Self::Translate(body, translate) => {
                body.eq(&[
                    1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
                ]) && translate.eq(&[0., 0., 0., 0.])
            }
        }
    }

    pub fn scale_only(&self) -> bool {
        match self {
            Self::Identity | Self::Scale(..) => true,
            Self::Translate(body, translate) => {
                body.iter().all(|v| *v == 0.) && translate.iter().all(|v| *v == 0.)
            }
        }
    }

    pub const fn scale_elemnts(self) -> (f32, f32, f32, f32) {
        match self {
            Self::Identity => (1., 1., 1., 1.),
            Self::Scale(v) => (v[0], v[1], v[2], v[3]),
            Self::Translate(body, ..) => (body[0], body[5], body[10], body[15]),
        }
    }

    pub const fn apply(self, r: u16, g: u16, b: u16, a: u16) -> (u16, u16, u16, u16) {
        let (rf, gf, bf, af) = Self::color_to_f32(r, g, b, a);

        match self {
            Self::Identity => (
                (rf * 65535.) as u16,
                (gf * 65535.) as u16,
                (bf * 65535.) as u16,
                (af * 65535.) as u16,
            ),
            Self::Scale(scale) => (
                (Self::clamp(rf * scale[0]) * 65535.) as u16,
                (Self::clamp(gf * scale[1]) * 65535.) as u16,
                (Self::clamp(bf * scale[2]) * 65535.) as u16,
                (Self::clamp(af * scale[3]) * 65535.) as u16,
            ),
            Self::Translate(body, translate) => (
                (Self::clamp(
                    rf * body[0] + gf * body[4] + bf * body[8] + af * body[12] + translate[0],
                ) * 65535.) as u16,
                (Self::clamp(
                    rf * body[1] + gf * body[5] + bf * body[9] + af * body[13] + translate[1],
                ) * 65535.) as u16,
                (Self::clamp(
                    rf * body[2] + gf * body[6] + bf * body[10] + af * body[14] + translate[2],
                ) * 65535.) as u16,
                (Self::clamp(
                    rf * body[3] + gf * body[7] + bf * body[11] + af * body[15] + translate[3],
                ) * 65535.) as u16,
            ),
        }
    }

    pub const fn elements(self) -> ([f32; 16], [f32; 4]) {
        match self {
            Self::Identity => (
                [
                    1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
                ],
                [0., 0., 0., 0.],
            ),
            Self::Scale(scale) => (
                [
                    scale[0], 0., 0., 0., 0., scale[1], 0., 0., 0., 0., scale[2], 0., 0., 0., 0.,
                    scale[3],
                ],
                [0., 0., 0., 0.],
            ),
            Self::Translate(body, translate) => (body, translate),
        }
    }

    pub fn is_invertible(&self) -> bool {
        match self {
            Self::Identity => true,
            Self::Scale(scale) => scale.iter().all(|&v| v != 0.),
            Self::Translate(body, ..) => Self::det(body) != 0.,
        }
    }

    pub const fn invert(self) -> Self {
        match self {
            Self::Identity => Self::Identity,
            Self::Scale(scale) => {
                Self::Scale([1. / scale[0], 1. / scale[1], 1. / scale[2], 1. / scale[3]])
            }
            Self::Translate(body, translate) => {
                let det = Self::det(&body);

                let m00 = body[0];
                let m01 = body[1];
                let m02 = body[2];
                let m03 = body[3];
                let m10 = body[4];
                let m11 = body[5];
                let m12 = body[6];
                let m13 = body[7];
                let m20 = body[8];
                let m21 = body[9];
                let m22 = body[10];
                let m23 = body[11];
                let m30 = body[12];
                let m31 = body[13];
                let m32 = body[14];
                let m33 = body[15];

                let m40 = translate[0];
                let m41 = translate[1];
                let m42 = translate[2];
                let m43 = translate[3];

                let a2334 = m32 * m43 - m33 * m42;
                let a1334 = m31 * m43 - m33 * m41;
                let a1234 = m31 * m42 - m32 * m41;
                let a0334 = m30 * m43 - m33 * m40;
                let a0234 = m30 * m42 - m32 * m40;
                let a0134 = m30 * m41 - m31 * m40;
                let a2324 = m22 * m43 - m23 * m42;
                let a1324 = m21 * m43 - m23 * m41;
                let a1224 = m21 * m42 - m22 * m41;
                let a0324 = m20 * m43 - m23 * m40;
                let a0224 = m20 * m42 - m22 * m40;
                let a0124 = m20 * m41 - m21 * m40;

                let b234234 = m22 * m33 - m23 * m32;
                let b134234 = m21 * m33 - m23 * m31;
                let b124234 = m21 * m32 - m22 * m31;
                let b123234 = m21 * a2334 - m22 * a1334 + m23 * a1234;
                let b034234 = m20 * m33 - m23 * m30;
                let b024234 = m20 * m32 - m22 * m30;
                let b023234 = m20 * a2334 - m22 * a0334 + m23 * a0234;
                let b014234 = m20 * m31 - m21 * m30;
                let b013234 = m20 * a1334 - m21 * a0334 + m23 * a0134;
                let b012234 = m20 * a1234 - m21 * a0234 + m22 * a0134;
                let b234134 = m12 * m33 - m13 * m32;
                let b134134 = m11 * m33 - m13 * m31;
                let b124134 = m11 * m32 - m12 * m31;
                let b123134 = m11 * a2334 - m12 * a1334 + m13 * a1234;
                let b234124 = m12 * m23 - m13 * m22;
                let b134124 = m11 * m23 - m13 * m21;
                let b124124 = m11 * m22 - m12 * m21;
                let b123124 = m11 * a2324 - m12 * a1324 + m13 * a1224;
                let b034134 = m10 * m33 - m13 * m30;
                let b024134 = m10 * m32 - m12 * m30;
                let b023134 = m10 * a2334 - m12 * a0334 + m13 * a0234;
                let b034124 = m10 * m23 - m13 * m20;
                let b024124 = m10 * m22 - m12 * m20;
                let b023124 = m10 * a2324 - m12 * a0324 + m13 * a0224;
                let b014134 = m10 * m31 - m11 * m30;
                let b013134 = m10 * a1334 - m11 * a0334 + m13 * a0134;
                let b014124 = m10 * m21 - m11 * m20;
                let b013124 = m10 * a1324 - m11 * a0324 + m13 * a0124;
                let b012134 = m10 * a1234 - m11 * a0234 + m12 * a0134;
                let b012124 = m10 * a1224 - m11 * a0224 + m12 * a0124;

                let idet = 1. / det;

                Self::Translate(
                    [
                        idet * (m11 * b234234 - m12 * b134234 + m13 * b124234),
                        idet * -(m01 * b234234 - m02 * b134234 + m03 * b124234),
                        idet * (m01 * b234134 - m02 * b134134 + m03 * b124134),
                        idet * -(m01 * b234124 - m02 * b134124 + m03 * b124124),
                        idet * -(m10 * b234234 - m12 * b034234 + m13 * b024234),
                        idet * (m00 * b234234 - m02 * b034234 + m03 * b024234),
                        idet * -(m00 * b234134 - m02 * b034134 + m03 * b024134),
                        idet * (m00 * b234124 - m02 * b034124 + m03 * b024124),
                        idet * (m10 * b134234 - m11 * b034234 + m13 * b014234),
                        idet * -(m00 * b134234 - m01 * b034234 + m03 * b014234),
                        idet * (m00 * b134134 - m01 * b034134 + m03 * b014134),
                        idet * -(m00 * b134124 - m01 * b034124 + m03 * b014124),
                        idet * -(m10 * b124234 - m11 * b024234 + m12 * b014234),
                        idet * (m00 * b124234 - m01 * b024234 + m02 * b014234),
                        idet * -(m00 * b124134 - m01 * b024134 + m02 * b014134),
                        idet * (m00 * b124124 - m01 * b024124 + m02 * b014124),
                    ],
                    [
                        idet * (m10 * b123234 - m11 * b023234 + m12 * b013234 - m13 * b012234),
                        idet * -(m00 * b123234 - m01 * b023234 + m02 * b013234 - m03 * b012234),
                        idet * (m00 * b123134 - m01 * b023134 + m02 * b013134 - m03 * b012134),
                        idet * -(m00 * b123124 - m01 * b023124 + m02 * b013124 - m03 * b012124),
                    ],
                )
            }
        }
    }

    pub const fn at(&self, i: usize, j: usize) -> f32 {
        match self {
            Self::Identity => {
                if i == j {
                    1.
                } else {
                    0.
                }
            }
            Self::Scale(scale) => {
                if i == j {
                    scale[i]
                } else {
                    0.
                }
            }
            Self::Translate(body, translate) => {
                if j < 4 {
                    body[i + j * 4]
                } else {
                    translate[i]
                }
            }
        }
    }

    pub fn set_elements(self, i: usize, j: usize, element: f32) -> Self {
        let (mut body, mut translate) = if !self.is_identity() {
            self.elements()
        } else {
            ([0.; 16], [0.; 4])
        };
        if j < 4 {
            body[i + j * 4] = element;
        } else {
            translate[i] = element;
        }

        Self::Translate(body, translate)
    }

    pub fn concat(self, other: Self) -> Self {
        match self {
            Self::Identity => other,
            Self::Scale(scale) => {
                if other.is_identity() {
                    self
                } else if other.scale_only() {
                    let (r, g, b, a) = other.scale_elemnts();
                    self.scale(r, g, b, a)
                } else {
                    let (body, translate) = other.elements();

                    Self::Translate(
                        [
                            body[0] * scale[0],
                            body[1] * scale[0],
                            body[2] * scale[0],
                            body[3] * scale[0],
                            body[4] * scale[1],
                            body[5] * scale[1],
                            body[6] * scale[1],
                            body[7] * scale[1],
                            body[8] * scale[2],
                            body[9] * scale[2],
                            body[10] * scale[2],
                            body[11] * scale[2],
                            body[12] * scale[3],
                            body[13] * scale[3],
                            body[14] * scale[3],
                            body[15] * scale[3],
                        ],
                        translate,
                    )
                }
            }
            Self::Translate(body, translate) => {
                if other.is_identity() {
                    self
                } else {
                    let (lbody, ltranslate) = other.elements();

                    Self::Translate(
                        Self::mul(&body, &lbody),
                        [
                            lbody[0] * translate[0]
                                + lbody[4] * translate[1]
                                + lbody[8] * translate[2]
                                + lbody[12] * translate[3]
                                + ltranslate[0],
                            lbody[1] * translate[0]
                                + lbody[5] * translate[1]
                                + lbody[9] * translate[2]
                                + lbody[13] * translate[3]
                                + ltranslate[1],
                            lbody[2] * translate[0]
                                + lbody[6] * translate[1]
                                + lbody[10] * translate[2]
                                + lbody[14] * translate[3]
                                + ltranslate[2],
                            lbody[3] * translate[0]
                                + lbody[7] * translate[1]
                                + lbody[11] * translate[2]
                                + lbody[15] * translate[3]
                                + ltranslate[3],
                        ],
                    )
                }
            }
        }
    }

    pub fn scale(self, r: f32, g: f32, b: f32, a: f32) -> Self {
        match self {
            Self::Identity => {
                if r == 1. && g == 1. && b == 1. && a == 1. {
                    self
                } else {
                    Self::Scale([r, g, b, a])
                }
            }
            Self::Scale(scale) => {
                if r == 1. && g == 1. && b == 1. && a == 1. {
                    self
                } else {
                    Self::Scale([r * scale[0], g * scale[1], b * scale[2], a * scale[3]])
                }
            }
            Self::Translate(mut body, mut translate) => {
                if r == 1. && g == 1. && b == 1. && a == 1. {
                    self
                } else if self.scale_only() {
                    let (sr, sg, sb, sa) = self.scale_elemnts();
                    Self::Scale([r * sr, g * sg, b * sb, a * sa])
                } else {
                    (0..4).for_each(|i| {
                        body[i * 4] *= r;
                        body[i * 4 + 1] *= g;
                        body[i * 4 + 2] *= b;
                        body[i * 4 + 3] *= a;
                    });
                    translate[0] += r;
                    translate[1] += g;
                    translate[2] += b;
                    translate[3] += a;

                    Self::Translate(body, translate)
                }
            }
        }
    }

    pub const fn translate(self, r: f32, g: f32, b: f32, a: f32) -> Self {
        if r == 0. && g == 0. && b == 0. && a == 0. {
            self
        } else {
            match self {
                Self::Identity => Self::Translate(
                    [
                        1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
                    ],
                    [r, g, b, a],
                ),
                Self::Scale(scale) => Self::Translate(
                    [
                        scale[0], 0., 0., 0., 0., scale[1], 0., 0., 0., 0., scale[2], 0., 0., 0.,
                        0., scale[3],
                    ],
                    [r, g, b, a],
                ),
                Self::Translate(body, translate) => Self::Translate(
                    body,
                    [
                        translate[0] + r,
                        translate[1] + g,
                        translate[2] + b,
                        translate[3] + a,
                    ],
                ),
            }
        }
    }

    pub fn change_hsv(self, hue_theta: f32, saturation_scale: f32, value_scale: f32) -> Self {
        if hue_theta == 0. && saturation_scale == 1. {
            self.scale(value_scale, value_scale, value_scale, 1.)
        } else {
            let (sin, cos) = hue_theta.sin_cos();

            self.concat(Self::Translate(
                [
                    0.2990, -0.1687, 0.5000, 0., 0.5870, -0.3313, -0.4187, 0., 0.1140, 0.5000,
                    -0.0813, 0., 0., 0., 0., 1.,
                ],
                [0.; 4],
            ))
            .concat(Self::Translate(
                [
                    1., 0., 0., 0., 0., cos, sin, 0., 0., -sin, cos, 0., 0., 0., 0., 1.,
                ],
                [0.; 4],
            ))
            .scale(
                value_scale,
                saturation_scale * value_scale,
                saturation_scale * value_scale,
                1.,
            )
            .concat(Self::Translate(
                [
                    1., 1., 1., 0., 0., -0.34414, 1.77200, 0., 1.40200, -0.71414, 0., 0., 0., 0.,
                    0., 1.,
                ],
                [0.; 4],
            ))
        }
    }

    pub(crate) fn to_scale(self) -> (Self, f32, f32, f32, f32) {
        if self.is_identity() || !self.scale_only() {
            (self, 1., 1., 1., 1.)
        } else {
            let r = self.at(0, 0);
            let g = self.at(1, 1);
            let b = self.at(2, 2);
            let a = self.at(3, 3);

            if r < 0. || g < 0. || b < 0. || a < 0. || r > 1. || g > 1. || b > 1. {
                (self, 1., 1., 1., 1.)
            } else {
                (Self::Identity, r * a, g * a, b * a, a)
            }
        }
    }

    const fn color_to_f32(r: u16, g: u16, b: u16, a: u16) -> (f32, f32, f32, f32) {
        if a > 0 {
            (
                r as f32 / a as f32,
                g as f32 / a as f32,
                b as f32 / a as f32,
                a as f32 / 65535.,
            )
        } else {
            (0., 0., 0., 0.)
        }
    }

    const fn clamp(x: f32) -> f32 {
        if x > 1. {
            1.
        } else if x < 0. {
            0.
        } else {
            x
        }
    }

    fn mul(lhs: &[f32; 16], rhs: &[f32; 16]) -> [f32; 16] {
        let mut result = [0.; 16];
        for i in 0..4 {
            for j in 0..4 {
                let mut elem = 0.;
                for k in 0..4 {
                    elem += lhs[i * 4 + k] * rhs[k * 4 + j];
                }
                result[i * 4 + j] = elem;
            }
        }

        result
    }

    const fn det(body: &[f32; 16]) -> f32 {
        let m00 = body[0];
        let m01 = body[1];
        let m02 = body[2];
        let m03 = body[3];
        let m10 = body[4];
        let m11 = body[5];
        let m12 = body[6];
        let m13 = body[7];
        let m20 = body[8];
        let m21 = body[9];
        let m22 = body[10];
        let m23 = body[11];
        let m30 = body[12];
        let m31 = body[13];
        let m32 = body[14];
        let m33 = body[15];

        let b234234 = m22 * m33 - m23 * m32;
        let b134234 = m21 * m33 - m23 * m31;
        let b124234 = m21 * m32 - m22 * m31;
        let b034234 = m20 * m33 - m23 * m30;
        let b024234 = m20 * m32 - m22 * m30;
        let b014234 = m20 * m31 - m21 * m30;

        m00 * (m11 * b234234 - m12 * b134234 + m13 * b124234)
            - m01 * (m10 * b234234 - m12 * b034234 * m13 * b024234)
            + m02 * (m10 * b134234 - m11 * b034234 + m13 * b014234)
            - m03 * (m10 * b124234 - m11 * b024234 + m12 * b014234)
    }
}

impl PartialEq for ColorM {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Identity => other.is_identity(),
            Self::Scale(scale) => {
                if !other.scale_only() {
                    return false;
                }
                let (r, g, b, a) = other.scale_elemnts();
                scale.eq(&[r, g, b, a])
            }
            Self::Translate(body, translate) => {
                let (lbody, ltranslate) = self.elements();
                body.eq(&lbody) && translate.eq(&ltranslate)
            }
        }
    }
}
