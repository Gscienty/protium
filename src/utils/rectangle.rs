#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl Point {
    pub(crate) const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub(crate) const fn in_region(&self, r: Rectangle) -> bool {
        r.min.x <= self.x && self.x < r.max.x && r.min.y <= self.y && self.y < r.max.y
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rectangle {
    pub(crate) min: Point,
    pub(crate) max: Point,
}

impl Rectangle {
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            min: Point::new(x, y),
            max: Point::new(x + width, y + height),
        }
    }

    pub const fn new_pos(x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
        let (x0, x1) = if x0 > x1 { (x1, x0) } else { (x0, x1) };
        let (y0, y1) = if y0 > y1 { (y1, y0) } else { (y0, y1) };
        Self {
            min: Point::new(x0, y0),
            max: Point::new(x1, y1),
        }
    }

    #[inline(always)]
    pub const fn get_width(&self) -> i32 {
        self.max.x - self.min.x
    }
    #[inline(always)]
    pub const fn get_height(&self) -> i32 {
        self.max.y - self.min.y
    }

    pub(crate) const fn is_empty(&self) -> bool {
        self.min.x >= self.max.x || self.min.y >= self.max.y
    }

    pub(crate) const fn is_in(&self, other: &Self) -> bool {
        if self.is_empty() {
            true
        } else {
            other.min.x <= self.min.x
                && self.max.x <= other.max.x
                && other.min.y <= self.min.y
                && self.max.y <= other.max.y
        }
    }

    pub const fn point_is_in(&self, x: i32, y: i32) -> bool {
        self.min.x <= x && x <= self.max.x && self.min.y <= y && y <= self.max.y
    }

    pub(crate) const fn add(&self, p: Point) -> Self {
        Self {
            min: Point {
                x: self.min.x + p.x,
                y: self.min.y + p.y,
            },
            max: Point {
                x: self.max.x + p.x,
                y: self.max.y + p.y,
            },
        }
    }

    pub(crate) const fn sub(&self, p: Point) -> Self {
        Self {
            min: Point {
                x: self.min.x - p.x,
                y: self.min.y - p.y,
            },
            max: Point {
                x: self.max.x - p.x,
                y: self.max.y - p.y,
            },
        }
    }

    pub(crate) const fn intersect(mut self, s: Self) -> Self {
        if self.min.x < s.min.x {
            self.min.x = s.min.x;
        }
        if self.min.y < s.min.y {
            self.min.y = s.min.y;
        }
        if self.max.x > s.max.x {
            self.max.x = s.max.x;
        }
        if self.max.y > s.max.y {
            self.max.y = s.max.y;
        }
        if self.is_empty() {
            Self::new(0, 0, 0, 0)
        } else {
            self
        }
    }

    pub(crate) const fn union(mut self, s: Self) -> Self {
        if self.is_empty() {
            return s;
        }
        if s.is_empty() {
            return self;
        }
        if self.min.x > s.min.x {
            self.min.x = s.min.x;
        }
        if self.min.y > s.min.y {
            self.min.y = s.min.y;
        }
        if self.max.x < s.max.x {
            self.max.x = s.max.x;
        }
        if self.max.y < s.max.y {
            self.max.y = s.max.y;
        }

        self
    }
}
