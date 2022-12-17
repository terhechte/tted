use forma::math::Point;

#[derive(Debug, Copy, Clone)]
pub struct Size {
    pub w: f32,
    pub h: f32,
}

impl From<(f32, f32)> for Size {
    fn from(tuple: (f32, f32)) -> Self {
        Size {
            w: tuple.0,
            h: tuple.1,
        }
    }
}

impl Size {
    pub const ZERO: Size = Size::new(0., 0.);

    pub const fn new(w: f32, h: f32) -> Self {
        Self { w, h }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}
