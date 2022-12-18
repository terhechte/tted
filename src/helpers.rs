use forma::prelude::{AffineTransform, Point};

pub trait AffineHelpers: Sized {
    fn from_raw(raw: &[f32; 9]) -> Self;
    fn inverse(self) -> Option<Self>;
    fn transform_point(self, point: Point) -> Point;
    fn new_mirror(x: bool, y: bool) -> Self;
    fn translated(&self, x: f32, y: f32) -> Self;
    fn scaled(&self, value: f32) -> Self;
    fn raw(&self) -> [f32; 9];
}

pub fn shift_raw_transform(transform: &[f32; 9], w: f32, h: f32) -> [f32; 9] {
    let mut original = *transform;
    original[2] += w;
    original[5] += h;
    original
}

impl AffineHelpers for AffineTransform {
    fn from_raw(raw: &[f32; 9]) -> Self {
        AffineTransform {
            ux: raw[0],
            vx: raw[1],
            uy: raw[3],
            vy: raw[4],
            tx: raw[2],
            ty: raw[5],
        }
    }

    fn inverse(self) -> Option<Self> {
        let det = self.ux * self.vy - self.vx * self.uy;
        if !det.is_finite() || det == 0. {
            return None;
        }
        let s = 1. / det;
        let a = self.ux;
        let b = self.uy;
        let c = self.vx;
        let d = self.vy;
        let x = self.tx;
        let y = self.ty;
        Some(AffineTransform {
            ux: d * s,
            uy: -b * s,
            vx: -c * s,
            vy: a * s,
            tx: (b * y - d * x) * s,
            ty: (c * x - a * y) * s,
        })
    }

    fn transform_point(self, point: Point) -> Point {
        Point {
            x: self.ux.mul_add(point.x, self.vx.mul_add(point.x, self.tx)),
            y: self.uy.mul_add(point.y, self.vy.mul_add(point.y, self.ty)),
        }
    }

    fn new_mirror(x: bool, y: bool) -> Self {
        AffineTransform {
            ux: x.then(|| -1.0).unwrap_or(1.0),
            uy: y.then(|| -1.0).unwrap_or(1.0),
            vx: 0.0,
            vy: 0.0,
            tx: 0.0,
            ty: 0.0,
        }
    }

    fn translated(&self, x: f32, y: f32) -> Self {
        let mut copy = *self;
        copy.tx += x * copy.ux;
        copy.ty += y * copy.vy;
        copy
    }

    fn scaled(&self, value: f32) -> Self {
        let mut copy = *self;
        copy.ux += value;
        copy.vy += value;
        copy
    }

    fn raw(&self) -> [f32; 9] {
        [
            self.ux, self.vx, self.tx, self.uy, self.vy, self.ty, 0.0, 0.0, 1.0,
        ]
    }
}
