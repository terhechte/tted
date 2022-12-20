use forma::prelude::Point;
use forma::styling::{Color, Fill, FillRule, Props, Style};
use forma::{prelude::AffineTransform, Composition};
use forma::{Order, PathBuilder};
use parley::FontContext;

use crate::helpers::AffineHelpers;
use crate::rich_text::{RichText, StyleProperty};
use crate::RunContext;
use crate::{
    layout_types::{Widget, WidgetContext},
    text::Text,
    types::Size,
};

pub struct Drawer {
    widget: Text,
    font_context: FontContext,
    transform: AffineTransform,
    needs_composition: bool,
    size: Size,
    debug_rect: bool,
    clip: bool,
}

impl Drawer {
    pub fn new() -> Self {
        let mut r = RichText::new([
            StyleProperty::Font("Archivo Black"),
            StyleProperty::FontSize(30.),
        ]);
        r.add_str("Headline 😀 💓 👨‍👩‍👦");
        r.add_newline();
        r.add_single("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut", StyleProperty::Font("Helvetica"));
        for _ in 0..5 {
            r.add_newline();
            r.add_single(include_str!("../LICENSE"), StyleProperty::Font("Helvetica"));
        }

        // r.add_str(include_str!("../emoji.txt"));

        let text = Text::new(r);

        let mut context = FontContext::new();

        const FONT_DATA: &[u8] = include_bytes!("ArchivoBlack-Regular.ttf");

        // FIXME: Move these into a separate type
        context.register_fonts(FONT_DATA.to_owned());

        let scale = 1.;
        let translate = 50.;
        let transform = AffineTransform {
            ux: scale,
            vx: 0.0,
            uy: 0.0,
            vy: scale,
            tx: translate * scale,
            ty: translate * scale,
        };

        Self {
            widget: text,
            transform,
            font_context: context,
            needs_composition: true,
            size: Size { w: 1000., h: 1000. },
            debug_rect: false,
            clip: false,
        }
    }
}

impl crate::App for Drawer {
    fn set_width(&mut self, width: usize) {
        self.size.w = width as f32;
    }

    fn set_height(&mut self, height: usize) {
        self.size.h = height as f32;
    }

    fn update(&mut self, context: &RunContext<'_>) {
        if let Some(delta) = context.mouse.wheel {
            self.transform = self.transform.scaled((delta.y / 1000.) as f32);
            self.needs_composition = true;
        }

        if context.mouse.pressed_left() {
            let delta = context.mouse.position_delta;
            self.transform = self.transform.translated(
                -delta.x as f32 * (1.0 / self.transform.ux),
                -delta.y as f32 * (1.0 / self.transform.vy),
            );
            self.needs_composition = true;
        }
    }

    fn compose<'a>(&mut self, composition: &mut Composition, context: &RunContext<'a>) {
        if !self.needs_composition {
            return;
        }

        let (w, h) = (self.size.w, self.size.h);

        let mut index = 2;

        let mut layout_context = WidgetContext {
            font_context: &mut self.font_context,
            transform: &self.transform,
            index: &mut index,
            clip: self.clip,
        };

        let size = Size::new(500., 5300.);

        // FIXME: We get size back here, do something with it?
        self.widget.layout(&mut layout_context, size);

        // Now we know how many layers to clip
        if self.clip {
            clip_rect(composition, 0, (*layout_context.index) as usize + 1, w, h);
        }

        self.widget
            .compose(&layout_context, composition, context.elapsed);

        if self.debug_rect {
            debug_rect(composition, 1, w, h);
        }

        self.needs_composition = false;
    }
}

fn clip_rect(composition: &mut Composition, index: u32, count: usize, w: f32, h: f32) {
    let mut builder = PathBuilder::new();
    builder.move_to(Point::new(0., 0.));
    builder.line_to(Point::new(w, 0.0));
    builder.line_to(Point::new(w, h));
    builder.line_to(Point::new(0., h));
    builder.line_to(Point::new(0., 0.));
    let path = builder.build();

    let layer = composition.get_mut_or_insert_default(Order::new(index).unwrap());
    layer.insert(&path).set_props(Props {
        fill_rule: FillRule::NonZero,
        func: forma::styling::Func::Clip(count),
    });
}

fn debug_rect(composition: &mut Composition, index: u32, w: f32, h: f32) {
    let mut builder = PathBuilder::new();
    builder.move_to(Point::new(0., 0.));
    builder.line_to(Point::new(w, 0.0));
    builder.line_to(Point::new(w, h));
    builder.line_to(Point::new(0., h));
    builder.line_to(Point::new(0., 0.));
    let path = builder.build();
    let debug_layer = composition.get_mut_or_insert_default(Order::new(index).unwrap());
    debug_layer.insert(&path).set_props(Props {
        fill_rule: FillRule::EvenOdd,
        func: forma::styling::Func::Draw(Style {
            is_clipped: false,
            fill: Fill::Solid(Color {
                r: 1.0,
                g: 0.8,
                b: 0.8,
                a: 0.8,
            }),
            ..Default::default()
        }),
    });
}
