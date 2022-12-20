use forma::math::{AffineTransform, Point};
use forma::styling::Image;
use forma::Path;
use parley::swash::scale::image::{Content, Image as SwashImage};
use parley::swash::zeno::Vector;
use parley::swash::zeno::{Bounds, Command};

use crate::helpers::AffineHelpers;

pub trait Convert {
    type Output;
    fn convert(self) -> Self::Output;
}

impl Convert for Vector {
    type Output = Point;

    fn convert(self) -> Self::Output {
        Point {
            x: self.x,
            y: self.y,
        }
    }
}

impl Convert for (f32, f32) {
    type Output = Point;

    fn convert(self) -> Self::Output {
        Point {
            x: self.0,
            y: self.1,
        }
    }
}

impl Convert for SwashImage {
    type Output = Option<Image>;
    fn convert(self) -> Self::Output {
        use image::{DynamicImage, RgbaImage};

        // we only support color bitmaps here
        if self.content != Content::Color {
            return None;
        }

        let rgba_image =
            RgbaImage::from_vec(self.placement.width, self.placement.height, self.data)?;
        let w = rgba_image.width();
        let h = rgba_image.height();

        let dynamic_image = DynamicImage::ImageRgba8(rgba_image);

        let data: Vec<_> = dynamic_image
            .to_rgba8()
            .pixels()
            .map(|p| [p.0[0], p.0[1], p.0[2], p.0[3]])
            .collect();
        Image::from_srgba(&data[..], w as usize, h as usize).ok()
    }
}

pub fn convert_path(
    value: impl Iterator<Item = Command>,
    transform: &AffineTransform,
) -> forma::Path {
    let mut builder = forma::PathBuilder::default();
    for entry in value {
        match entry {
            Command::MoveTo(p) => {
                builder.move_to(transform.transform_point(p.convert()));
            }
            Command::LineTo(p) => {
                builder.line_to(transform.transform_point(p.convert()));
            }
            Command::QuadTo(p1, p2) => {
                builder.quad_to(
                    transform.transform_point(p1.convert()),
                    transform.transform_point(p2.convert()),
                );
            }
            Command::CurveTo(p1, p2, p3) => {
                builder.cubic_to(
                    transform.transform_point(p1.convert()),
                    transform.transform_point(p2.convert()),
                    transform.transform_point(p3.convert()),
                );
            }
            Command::Close => {}
        }
    }
    builder.build()
}

pub fn convert_bounds(bounds: &Bounds, transform: &AffineTransform) -> Path {
    fn convert(x: f32, y: f32, t: &AffineTransform) -> Point {
        t.transform_point((x, y).convert())
    }
    let mut builder = forma::PathBuilder::default();
    builder.move_to(convert(bounds.min.x, bounds.min.y, transform));
    builder.line_to(convert(
        bounds.min.x + bounds.max.x,
        bounds.min.y,
        transform,
    ));
    builder.line_to(convert(
        bounds.min.x + bounds.max.x,
        bounds.min.y + bounds.max.y,
        transform,
    ));
    builder.line_to(convert(
        bounds.min.x,
        bounds.min.y + bounds.max.y,
        transform,
    ));
    builder.line_to(convert(bounds.min.x, bounds.min.y, transform));
    builder.build()
}
