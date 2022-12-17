use std::time::Duration;

use forma::{
    math::GeomPresTransform,
    prelude::{AffineTransform, Point},
    styling::{Color, Fill, FillRule, Func, Props, Style},
    Composition, Order,
};
use pinot::{FontRef, TableProvider};

use crate::Keyboard;

pub struct Drawer {
    font_ref: FontRef<'static>,
    text: String,
    needs_composition: bool,
}

impl Drawer {
    pub fn new() -> Self {
        //const FONT_DATA: &[u8] = include_bytes!("Roboto-Regular.ttf");
        const FONT_DATA: &[u8] = include_bytes!("ArchivoBlack-Regular.ttf");
        let font = FontRef {
            data: FONT_DATA,
            offset: 0,
        };

        Self {
            font_ref: font,
            text: "Hello World".to_string(),
            needs_composition: true,
        }
    }
}

impl crate::App for Drawer {
    fn set_width(&mut self, width: usize) {}

    fn set_height(&mut self, height: usize) {}

    fn compose(&mut self, composition: &mut Composition, elapsed: Duration, keyboard: &Keyboard) {
        if !self.needs_composition {
            return;
        }

        let font = self.font_ref;
        let transform = GeomPresTransform::default();
        dbg!("render");

        let mut context = moscato::Context::new();
        if let Some(cmap) = font.cmap() {
            if let Some(hmtx) = font.hmtx() {
                let layer = composition
                    .get_mut_or_insert_default(Order::new(0 as u32).unwrap())
                    .clear();

                let upem = font.head().map(|head| head.units_per_em()).unwrap_or(1000) as f32;
                let scale = 34.0 as f32 / upem;
                let vars: [(pinot::types::Tag, f32); 0] = [];

                let mut scaler = context
                    .new_scaler_with_id(&font, 1)
                    .size(34.0)
                    .hint(true)
                    .variations(vars)
                    .build();

                let hmetrics = hmtx.hmetrics();
                let default_advance = hmetrics
                    .get(hmetrics.len().saturating_sub(1))
                    .map(|h| h.advance_width)
                    .unwrap_or(0);
                let mut pen_x = 51f32;
                let mut pen_y = 51f32;

                // The mirror transform
                let transform = AffineTransform {
                    ux: 1.0,
                    uy: -1.0,
                    vx: 0.0,
                    vy: 0.0,
                    tx: 0.0,
                    ty: 0.0,
                };

                for ch in self.text.chars() {
                    let gid = cmap.map(ch as u32).unwrap_or(0);
                    let advance = hmetrics
                        .get(gid as usize)
                        .map(|h| h.advance_width)
                        .unwrap_or(default_advance) as f32
                        * scale;

                    if let Some(glyph) = scaler.glyph(gid) {
                        if let Some(path) = glyph.path(0) {
                            let path = c_path(path, &transform);
                            layer
                                .insert(
                                    // https://tinylittlemaggie.github.io/transformation-matrix-playground/
                                    &path.transform(&[
                                        1.0, 0.0, pen_x, 0.0, 1.0, pen_y, 0.0, 0.0, 1.0,
                                    ]),
                                )
                                .set_props(Props {
                                    fill_rule: FillRule::NonZero,
                                    func: Func::Draw(Style {
                                        fill: Fill::Solid(Color {
                                            r: 0.0,
                                            g: 0.0,
                                            b: 0.0,
                                            a: 1.0,
                                        }),
                                        ..Default::default()
                                    }),
                                });
                        }

                        // FIXME: this is where we would draw the text

                        // let xform = transform
                        //     * Affine::translate((pen_x, 0.0))
                        //     * Affine::scale_non_uniform(1.0, -1.0);
                        // dbg!(path);
                        // dbg!(xform);

                        // provider.get(gid, brush) {
                        //     let xform = transform
                        //         * Affine::translate((pen_x, 0.0))
                        //         * Affine::scale_non_uniform(1.0, -1.0);
                        //     builder.append(&glyph, Some(xform));
                    }
                    pen_x += advance;
                }
            }
        }

        self.needs_composition = false;
    }
}

fn c_path(value: moscato::Path, transform: &AffineTransform) -> forma::Path {
    let mut builder = forma::PathBuilder::default();
    for entry in value.elements() {
        match entry {
            moscato::Element::MoveTo(p) => {
                builder.move_to(c_point(p, transform));
            }
            moscato::Element::LineTo(p) => {
                builder.line_to(c_point(p, transform));
            }
            moscato::Element::QuadTo(p1, p2) => {
                builder.quad_to(c_point(p1, transform), c_point(p2, transform));
            }
            moscato::Element::CurveTo(p1, p2, p3) => {
                builder.cubic_to(
                    c_point(p1, transform),
                    c_point(p2, transform),
                    c_point(p3, transform),
                );
            }
            moscato::Element::Close => {}
        }
    }
    builder.build()
}

fn c_point(value: moscato::Point, tf: &AffineTransform) -> forma::prelude::Point {
    Point {
        x: tf.ux.mul_add(value.x, tf.vx.mul_add(value.x, tf.tx)),
        y: tf.uy.mul_add(value.y, tf.vy.mul_add(value.y, tf.ty)),
    }
}
