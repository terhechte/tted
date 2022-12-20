use std::time::Duration;

use crate::conversion::{convert_bounds, convert_path, Convert};
use crate::helpers::{shift_raw_transform, AffineHelpers};
use crate::layout_types::{Widget, WidgetContext};
use crate::rich_text::RichText;
use crate::types::Size;

use emoji::lookup_by_glyph::lookup;
use forma::prelude::*;
use parley::swash::scale::ScaleContext;
use parley::swash::scale::StrikeWith;
use parley::swash::zeno::PathData;

#[derive(Default)]
struct GlyphRunCache {
    layer_id: u32,
    glyphs: Vec<GlyphCache>,
}

enum GlyphCache {
    Text {
        path: Path,
        style: Style,
        point: Point,
    },
    Bitmap {
        path: Path,
        image: Image,
        height: f32,
        point: Point,
    },
}

pub struct Text {
    text: RichText,
    cache: Vec<GlyphRunCache>,
    cached_size: Size,
    needs_layout: bool,
}

impl Text {
    pub fn new(text: RichText) -> Self {
        let capacity = text.attribute_count() + text.attribute_count() / 2;
        Self {
            text,
            cache: Vec::with_capacity(capacity),
            cached_size: Size::ZERO,
            needs_layout: true,
        }
    }

    pub fn update(&mut self, text: RichText) {
        self.text = text;
        self.needs_layout = true;
        self.cache.clear();
        self.cached_size = Size::ZERO;
    }
}

impl Widget for Text {
    fn layout<'a>(&mut self, ctx: &mut WidgetContext<'a>, proposed_size: Size) -> Size {
        if !self.needs_layout {
            return self.cached_size;
        }
        let mut layout_context = parley::LayoutContext::new();
        let mut layout = self.text.build(&mut layout_context, ctx.font_context);
        layout.break_all_lines(Some(proposed_size.w), parley::layout::Alignment::Start);

        // The mirror transform for individual characters
        let transform = AffineTransform::new_mirror(false, true);

        let size = (layout.width(), layout.height()).into();

        let mut context = ScaleContext::new();

        for line in layout.lines() {
            for glyph_run in line.glyph_runs() {
                // each run needs a new layer as a run distinguishes colors (logic here can probably be simplified)
                let layer_id = *ctx.index;
                *ctx.index += 1;

                let mut glyph_cache = GlyphRunCache {
                    layer_id,
                    ..Default::default()
                };

                let mut x = glyph_run.offset();
                let y = glyph_run.baseline();
                let run = glyph_run.run();
                let font = run.font().as_ref();
                let font_size = run.font_size();
                let style = glyph_run.style();
                let vars: [(parley::swash::Tag, f32); 0] = [];

                let range = run.text_range();
                let slice = &self.text.slice(range);

                let mut scaler = context
                    .builder(font)
                    .hint(true)
                    .size(font_size)
                    .hint(false)
                    .variations(vars)
                    .build();

                for glyph in glyph_run.glyphs() {
                    let is_emoji = lookup(slice).is_some();

                    let Some(outline) = scaler.scale_outline(glyph.id) else {
                                x += glyph.advance;
                                continue
                            };
                    if let Some(image) = is_emoji
                        .then(|| {
                            scaler
                                .scale_color_bitmap(glyph.id, StrikeWith::BestFit)
                                .and_then(|img| img.convert())
                        })
                        .flatten()
                    {
                        let bounds = outline.bounds();
                        let path = convert_bounds(&bounds, &transform);

                        glyph_cache.glyphs.push(GlyphCache::Bitmap {
                            path,
                            image,
                            height: bounds.height(),
                            point: Point::new(x, y),
                        });
                    } else {
                        let path = convert_path(outline.path().commands(), &transform);

                        glyph_cache.glyphs.push(GlyphCache::Text {
                            path,
                            style: Style {
                                is_clipped: ctx.clip,
                                fill: style.brush.fill.clone(),
                                ..Default::default()
                            },
                            point: Point::new(x, y),
                        });
                    }
                    x += glyph.advance;
                }

                self.cache.push(glyph_cache);
            }
        }

        self.cached_size = size;
        self.needs_layout = false;

        size
    }

    fn compose<'a>(
        &mut self,
        ctx: &WidgetContext<'a>,
        composition: &mut Composition,
        _elapsed: Duration,
    ) {
        for entry in self.cache.iter() {
            let layer = composition
                .get_mut_or_insert_default(Order::new(entry.layer_id).unwrap())
                .clear();
            for glyph in entry.glyphs.iter() {
                match glyph {
                    GlyphCache::Text { path, style, point } => {
                        let path_transform = ctx.transform.translated(point.x, point.y);
                        let path = path.transform(&path_transform.raw());
                        layer.insert(&path).set_props(Props {
                            fill_rule: FillRule::NonZero,
                            func: Func::Draw(style.clone()),
                        });
                    }
                    GlyphCache::Bitmap {
                        path,
                        image,
                        height,
                        point,
                    } => {
                        let path_transform = ctx.transform.translated(point.x, point.y);
                        let path = path.transform(&path_transform.raw());
                        let texture_transform = AffineTransform::from_raw(&shift_raw_transform(
                            &path_transform.raw(),
                            0.0,
                            -height * ctx.transform.vy,
                        ))
                        .inverse()
                        .unwrap_or_default();

                        layer.insert(&path).set_props(Props {
                            fill_rule: FillRule::NonZero,
                            func: Func::Draw(Style {
                                is_clipped: ctx.clip,
                                fill: Fill::Texture(forma::styling::Texture {
                                    transform: texture_transform,
                                    image: image.clone(),
                                }),
                                ..Default::default()
                            }),
                        });
                    }
                }
            }
        }
    }
}
