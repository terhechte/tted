use std::time::Duration;

use crate::layout_types::{FormaBrush, LayoutContext, Widget};
use crate::types::Size;

use forma::prelude::*;
use parley::style::{FontFamily, FontStack};
use parley::Layout;
use pinot::types::Tag;

pub struct Text {
    text: String,
    layout: Option<Layout<FormaBrush>>,
    is_wrapped: bool,
    needs_update: bool,
}

impl Text {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            layout: None,
            is_wrapped: false,
            needs_update: true,
        }
    }
}

impl Widget for Text {
    fn measure(&mut self) -> (Size, Size) {
        (Size::ZERO, Size::new(50., 50.))
    }

    fn layout<'a>(&mut self, ctx: &mut LayoutContext<'a>, proposed_size: Size) -> Size {
        let mut lcx = parley::LayoutContext::new();
        let mut layout_builder = lcx.ranged_builder(ctx.font_context, &self.text, 1.0);
        layout_builder.push_default(&parley::style::StyleProperty::Brush(FormaBrush::default()));
        layout_builder.push_default(&parley::style::StyleProperty::FontStack(FontStack::Single(
            FontFamily::Named("Archivo Black"),
        )));
        let mut layout = layout_builder.build();
        layout.break_all_lines(None, parley::layout::Alignment::Start);
        let size = (layout.width(), layout.height()).into();
        self.layout = Some(layout);
        self.needs_update = true;
        size
    }

    fn compose<'a>(
        &mut self,
        ctx: &LayoutContext<'a>,
        composition: &mut Composition,
        elapsed: Duration,
    ) {
        if !self.needs_update {
            return;
        }
        self.needs_update = false;

        // FIXME: Replace with the transform in ctx
        let uniscale = 10f32;
        let unitranslate = (-50f32, -50f32);

        // The mirror transform for individual characters
        let transform = AffineTransform {
            ux: 1.0,
            uy: -1.0,
            vx: 0.0,
            vy: 0.0,
            tx: 0.0,
            ty: 0.0,
        };

        let Some(layout) = self.layout.as_ref() else { return };

        let mut context = moscato::Context::new();

        for line in layout.lines() {
            for glyph_run in line.glyph_runs() {
                let layer = composition
                    .get_mut_or_insert_default(Order::new(ctx.index as u32).unwrap())
                    .clear();

                let mut x = glyph_run.offset();
                let y = glyph_run.baseline();
                let run = glyph_run.run();
                let font = run.font().as_ref();
                let font_size = run.font_size();
                dbg!(&font.key);
                let font_ref = pinot::FontRef {
                    data: font.data,
                    offset: font.offset,
                };
                let style = glyph_run.style();
                let vars: [(Tag, f32); 0] = [];

                //let mut gp = gcx.new_provider(&font_ref, None, font_size, false, vars);

                let mut scaler = context
                    .new_scaler(&font_ref)
                    .size(font_size)
                    .hint(false)
                    .variations(vars)
                    .build();

                for glyph in glyph_run.glyphs() {
                    if let Some(g) = scaler.glyph(glyph.id) {
                        if let Some(path) = g.path(0) {
                            let path = c_path(path, &transform);
                            layer
                                .insert(
                                    // https://tinylittlemaggie.github.io/transformation-matrix-playground/
                                    &path.transform(&[
                                        uniscale,
                                        0.0,
                                        x * uniscale + unitranslate.0,
                                        0.0,
                                        uniscale,
                                        y * uniscale + unitranslate.1,
                                        0.0,
                                        0.0,
                                        1.0,
                                    ]),
                                )
                                .set_props(Props {
                                    fill_rule: FillRule::NonZero,
                                    func: Func::Draw(Style {
                                        fill: style.brush.fill.clone(),
                                        ..Default::default()
                                    }),
                                });
                        }
                        x += glyph.advance;
                    }
                    // if let Some(fragment) = gp.get(glyph.id, Some(&style.brush.0)) {
                    //     let gx = x + glyph.x;
                    //     let gy = y - glyph.y;
                    //     let xform = Affine::translate((gx as f64, gy as f64))
                    //         * Affine::scale_non_uniform(1.0, -1.0);
                    //     builder.append(&fragment, Some(transform * xform));
                    // }
                    // x += glyph.advance;
                }
            }
        }
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
