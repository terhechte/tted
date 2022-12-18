use std::time::Duration;

use crate::layout_types::{CacheKey, FormaBrush, LayoutContext, Widget};
use crate::types::Size;

use emoji::lookup_by_glyph::lookup;
use emoji::Emoji;
use forma::prelude::*;
use moscato::Transform;
use parley::style::{FontFamily, FontStack};
use parley::swash::scale::image::{Content as SwashContent, Image as SwashImage};
use parley::swash::scale::{Render, Source, StrikeWith};
use parley::swash::scale::{ScaleContext, Scaler};
use parley::swash::zeno::{Format, PathData};
use parley::{FontContext, Layout};
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
        layout_builder.push_default(&parley::style::StyleProperty::FontSize(30.));
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
        ctx: &mut LayoutContext<'a>,
        composition: &mut Composition,
        elapsed: Duration,
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

        // let mut context = moscato::Context::new();
        let mut context = ScaleContext::new();

        // FIXME: That clear makes the whole screen black for emoji.. why.. AH BECAUSE IT IS A NEW RUN?

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
                // let font_ref = pinot::FontRef {
                //     data: font.data,
                //     offset: font.offset,
                // };
                let style = glyph_run.style();
                let vars: [(parley::swash::Tag, f32); 0] = [];

                let range = run.text_range();
                let slice = &self.text[range];

                //let mut gp = gcx.new_provider(&font_ref, None, font_size, false, vars);

                // let mut scaler = context
                //     .new_scaler(&font_ref)
                //     .size(font_size)
                //     .hint(false)
                //     .variations(vars)
                //     .build();
                let mut scaler = context
                    .builder(font)
                    .hint(true)
                    .size(font_size)
                    .hint(false)
                    .variations(vars)
                    .build();

                for glyph in glyph_run.glyphs() {
                    // if let Some(emoji) =
                    //     lookup(slice).and_then(|emoji| swash_image(&mut scaler, glyph.id))
                    // {
                    //     //
                    // } else

                    // let glyph = scaler.glyph(glyph.id); // old style

                    // FIXME: Cache
                    let is_emoji = lookup(slice).is_some();
                    println!("is emoji {is_emoji} {slice:?}");

                    if is_emoji {
                        if let Some(image) = swash_image(&mut scaler, glyph.id, uniscale) {
                            // Shadow the affine transform above, we don't want to mirror here
                            // let transform = AffineTransform::default();
                            let mut builder = forma::PathBuilder::default();
                            let p = image.placement;
                            builder.move_to(c_point3(p.left, p.top - p.height as i32, &transform));
                            builder.line_to(c_point3(
                                p.left + p.width as i32,
                                p.top - p.height as i32,
                                &transform,
                            ));
                            builder.line_to(c_point3(p.left + p.width as i32, p.top, &transform));
                            builder.line_to(c_point3(p.left, p.top, &transform));
                            let path = builder.build();

                            let tx = &[
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

                            if let Some(ix) = c_image(image) {
                                let outline = scaler.scale_outline(glyph.id);

                                if let Some(g) = outline {
                                    dbg!(g.bounds());
                                    dbg!(glyph.x, glyph.y);
                                }
                                // println!("a {}", 1.0 / uniscale);
                                // println!("b {}", ix.width());
                                // println!("c {}", p.width);
                                // dbg!(&ix);

                                let tx2 = &[
                                    uniscale,
                                    0.0,
                                    x * uniscale + unitranslate.0,
                                    0.0,
                                    uniscale,
                                    y * uniscale + unitranslate.1 - ix.height() as f32,
                                    0.0,
                                    0.0,
                                    1.0,
                                ];

                                layer
                                    .insert(
                                        // https://tinylittlemaggie.github.io/transformation-matrix-playground/
                                        &path.transform(tx),
                                    )
                                    .set_props(Props {
                                        fill_rule: FillRule::NonZero,
                                        func: Func::Draw(Style {
                                            fill: Fill::Texture(forma::styling::Texture {
                                                // transform: AffineTransform {
                                                //     ux: 1.0 / 5.0,
                                                //     uy: 0.0,
                                                //     vx: 0.0,
                                                //     vy: 1.0 / 5.0,
                                                //     tx: 0.0,
                                                //     ty: 0.0,
                                                // },
                                                transform: inverse(&tx2).unwrap(),
                                                image: ix,
                                            }),
                                            ..Default::default()
                                        }),
                                    });
                            } else {
                                println!("Notin");
                            }

                            // dbg!(&image.content);
                            // dbg!(&image.placement);
                            // let outline = scaler.scale_outline(glyph.id);
                            // dbg!(outline.as_ref().map(|e| e.bounds()));
                            // dbg!(outline.as_ref().map(|e| c_path2(e.path(), &transform)));
                        }
                        x += font_size;
                        // layer.insert(&Path::default());
                        continue;
                    } else {
                        let outline = scaler.scale_outline(glyph.id);

                        if let Some(g) = outline {
                            // dbg!(c, &g);
                            let path = c_path2(g.path(), &transform);
                            // dbg!(&path);
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
                            //x += glyph.advance;
                            x += glyph.advance;
                        }
                    }
                }
            }
        }
    }
}

fn c_path2(value: impl PathData, transform: &AffineTransform) -> forma::Path {
    let mut builder = forma::PathBuilder::default();
    use parley::swash::zeno::Command;
    for entry in value.commands() {
        match entry {
            Command::MoveTo(p) => {
                builder.move_to(c_point2(p, transform));
            }
            Command::LineTo(p) => {
                builder.line_to(c_point2(p, transform));
            }
            Command::QuadTo(p1, p2) => {
                builder.quad_to(c_point2(p1, transform), c_point2(p2, transform));
            }
            Command::CurveTo(p1, p2, p3) => {
                builder.cubic_to(
                    c_point2(p1, transform),
                    c_point2(p2, transform),
                    c_point2(p3, transform),
                );
            }
            Command::Close => {}
        }
    }
    builder.build()
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

fn c_point2(value: parley::swash::zeno::Vector, tf: &AffineTransform) -> forma::prelude::Point {
    Point {
        x: tf.ux.mul_add(value.x, tf.vx.mul_add(value.x, tf.tx)),
        y: tf.uy.mul_add(value.y, tf.vy.mul_add(value.y, tf.ty)),
    }
}

fn c_point(value: moscato::Point, tf: &AffineTransform) -> forma::prelude::Point {
    Point {
        x: tf.ux.mul_add(value.x, tf.vx.mul_add(value.x, tf.tx)),
        y: tf.uy.mul_add(value.y, tf.vy.mul_add(value.y, tf.ty)),
    }
}

fn c_point3(x: i32, y: i32, tf: &AffineTransform) -> forma::prelude::Point {
    let x = x as f32;
    let y = y as f32;
    Point {
        x: tf.ux.mul_add(x, tf.vx.mul_add(x, tf.tx)),
        y: tf.uy.mul_add(y, tf.vy.mul_add(y, tf.ty)),
    }
}

// - render a given emoji
// - into a cache
// - return the cache value if available
// - above: render the image as a texture instead of a path (or rather, a path with a texture)
fn swash_image(scaler: &mut Scaler, glyph_id: u16, scale: f32) -> Option<SwashImage> {
    dbg!(scale);
    use parley::swash::zeno::Transform;
    // use parley::swash::scale::
    // let font = match font_system.get_font(cache_key.font_id) {
    //     Some(some) => some,
    //     None => {
    //         log::warn!("did not find font {:?}", cache_key.font_id);
    //         return None;
    //     }
    // };

    // // Build the scaler
    // let mut scaler = context
    //     .builder(font.as_swash())
    //     .size(cache_key.font_size as f32)
    //     .hint(true)
    //     .build();

    // Compute the fractional offset-- you'll likely want to quantize this
    // in a real renderer
    // let offset = Vector::new(cache_key.x_bin.as_float(), cache_key.y_bin.as_float());

    // Select our source order
    Render::new(&[
        // Color outline with the first palette
        Source::ColorOutline(0),
        // Color bitmap with best fit selection mode
        Source::ColorBitmap(StrikeWith::BestFit),
        // Standard scalable outline
        Source::Outline,
    ])
    // Select a subpixel format
    .format(Format::Alpha)
    // Apply the fractional offset
    // FIXME:
    // .offset(offset)
    // Render the image
    .render(scaler, glyph_id)
}

fn c_image(input: SwashImage) -> Option<forma::styling::Image> {
    use image::{io::Reader, DynamicImage, ImageBuffer, RgbaImage};
    use std::io::Cursor;
    // we only support color bitmaps here
    if input.content != SwashContent::Color {
        println!("Invalid image");
        return None;
    }

    let x = RgbaImage::from_vec(input.placement.width, input.placement.height, input.data)?;
    let w = x.width();
    let h = x.height();

    let dx = DynamicImage::ImageRgba8(x);
    // dx.save_with_format("/tmp/emoji.png", image::ImageFormat::Png)
    //     .unwrap();

    let data: Vec<_> = dx
        .to_rgba8()
        .pixels()
        .map(|p| [p.0[0], p.0[1], p.0[2], p.0[3]])
        .collect();
    let img = Image::from_srgba(&data[..], w as usize, h as usize).unwrap();

    // img.

    Some(img)

    // let r = Reader::new(Cursor::new(input.data))
    //     .with_guessed_format()
    //     .ok()?
    //     .decode();
    // match r {
    //     Ok(n) => {
    //         println!("format: {:?}", &n);
    //     }
    //     _ => return None,
    // }
    // None
    // let image = DynamicImage::
    // let data: Vec<_> = input
    //     .data
    //     .to_rgb8()
    //     .pixels()
    //     .map(|p| [p.0[0], p.0[1], p.0[2], 255])
    //     .collect();
    // Image::from_srgba(
    //     &data[..],
    //     input.placement.width as usize,
    //     input.placement.height as usize,
    // )
}

fn inverse(transform: &[f32; 9]) -> Option<AffineTransform> {
    let affine = AffineTransform {
        ux: transform[0],
        vx: transform[1],
        uy: transform[3],
        vy: transform[4],
        tx: transform[2],
        ty: transform[5],
    };
    let det = affine.ux * affine.vy - affine.vx * affine.uy;
    if !det.is_finite() || det == 0. {
        return None;
    }
    let s = 1. / det;
    let a = affine.ux;
    let b = affine.uy;
    let c = affine.vx;
    let d = affine.vy;
    let x = affine.tx;
    let y = affine.ty;
    Some(AffineTransform {
        ux: d * s,
        uy: -b * s,
        vx: -c * s,
        vy: a * s,
        tx: (b * y - d * x) * s,
        ty: (c * x - a * y) * s,
    })
}
