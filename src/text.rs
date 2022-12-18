use std::time::Duration;

use crate::conversion::{convert_bounds, convert_path, Convert};
use crate::helpers::{shift_raw_transform, AffineHelpers};
use crate::layout_types::{FormaBrush, LayoutContext, Widget};
use crate::types::Size;

use emoji::lookup_by_glyph::lookup;
use forma::prelude::*;
use parley::style::{FontFamily, FontStack};
use parley::swash::scale::{Render, Source, StrikeWith};
use parley::swash::scale::{ScaleContext, Scaler};
use parley::swash::zeno::{Format, PathData};
use parley::Layout;

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
    fn layout<'a>(&mut self, ctx: &mut LayoutContext<'a>, proposed_size: Size) -> Size {
        let mut lcx = parley::LayoutContext::new();
        let mut layout_builder = lcx.ranged_builder(ctx.font_context, &self.text, 1.0);
        layout_builder.push_default(&parley::style::StyleProperty::Brush(FormaBrush::default()));
        layout_builder.push_default(&parley::style::StyleProperty::FontSize(30.));
        layout_builder.push_default(&parley::style::StyleProperty::FontStack(FontStack::Single(
            FontFamily::Named("Archivo Black"),
        )));
        layout_builder.push(
            &parley::style::StyleProperty::FontStack(FontStack::Single(FontFamily::Named(
                "Helvetica",
            ))),
            0..3,
        );
        let mut layout = layout_builder.build();
        layout.break_all_lines(Some(proposed_size.w), parley::layout::Alignment::Start);
        let size = (layout.width(), layout.height()).into();
        self.layout = Some(layout);
        self.needs_update = true;
        size
    }

    fn compose<'a>(
        &mut self,
        ctx: &mut LayoutContext<'a>,
        composition: &mut Composition,
        _elapsed: Duration,
    ) {
        // FIXME: Move all this into the layout function? and just keep a vec of paths + brushes/fills?
        if !self.needs_update {
            return;
        }
        self.needs_update = false;

        // FIXME: Replace with the transform in ctx
        let uniscale = 1f32;
        let unitranslate = (50f32, 50f32);

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

        let mut context = ScaleContext::new();

        for line in layout.lines() {
            for glyph_run in line.glyph_runs() {
                // each run needs a new layer as a run distinguishes colors (logic here can probably be simplified)
                let layer = composition
                    .get_mut_or_insert_default(Order::new(*ctx.index as u32).unwrap())
                    .clear();
                *ctx.index += 1;

                let mut x = glyph_run.offset();
                let y = glyph_run.baseline();
                let run = glyph_run.run();
                let font = run.font().as_ref();
                let font_size = run.font_size();
                let style = glyph_run.style();
                let vars: [(parley::swash::Tag, f32); 0] = [];

                let range = run.text_range();
                let slice = &self.text[range];

                let mut scaler = context
                    .builder(font)
                    .hint(true)
                    .size(font_size)
                    .hint(false)
                    .variations(vars)
                    .build();

                for glyph in glyph_run.glyphs() {
                    let is_emoji = lookup(slice).is_some();

                    // FIXME: Refactor into .translate, .scale, etc functions for raw affines
                    let path_transform = &[
                        uniscale,
                        0.0,
                        x * uniscale + unitranslate.0,
                        0.0,
                        uniscale,
                        y * uniscale + unitranslate.1,
                        0.0,
                        0.0,
                        1.0,
                    ];

                    // FIXME: Check if the color_outline or has_bitmap things tell me whether
                    // it is a emoji (so I can remove the is_emoji check)

                    let Some(outline) = scaler.scale_outline(glyph.id) else {
                        x += glyph.advance;
                        continue
                    };

                    if let Some(image) = is_emoji
                        .then(|| render_image(&mut scaler, glyph.id))
                        .flatten()
                    {
                        let bounds = outline.bounds();
                        let path = convert_bounds(&bounds, &transform);

                        let texture_transform = AffineTransform::from_raw(&shift_raw_transform(
                            path_transform,
                            0.0,
                            -bounds.height(),
                        ))
                        .inverse()
                        .unwrap_or_default();

                        layer
                            .insert(&path.transform(path_transform))
                            .set_props(Props {
                                fill_rule: FillRule::NonZero,
                                func: Func::Draw(Style {
                                    fill: Fill::Texture(forma::styling::Texture {
                                        transform: texture_transform,
                                        image,
                                    }),
                                    ..Default::default()
                                }),
                            });
                    } else {
                        let path = convert_path(outline.path().commands(), &transform);
                        layer
                            .insert(&path.transform(path_transform))
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
            }
        }
    }
}

fn render_image(scaler: &mut Scaler, glyph_id: u16) -> Option<forma::styling::Image> {
    let image = Render::new(&[
        Source::ColorOutline(0),
        Source::ColorBitmap(StrikeWith::BestFit),
        Source::Outline,
    ])
    .format(Format::Alpha)
    .render(scaler, glyph_id)?;
    image.convert()
}
