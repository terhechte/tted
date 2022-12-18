use std::time::Duration;

use forma::{prelude::AffineTransform, Composition};
use parley::FontContext;

use crate::helpers::AffineHelpers;
use crate::rich_text::{RichText, StyleProperty};
use crate::{
    layout_types::{Widget, WidgetContext},
    text::Text,
    types::Size,
};
use crate::{Keyboard, RunContext};

pub struct Drawer {
    widget: Text,
    font_context: FontContext,
    transform: AffineTransform,
    needs_composition: bool,
    size: Size,
}

impl Drawer {
    pub fn new() -> Self {
        let mut r = RichText::new([
            StyleProperty::Font("Archivo Black"),
            StyleProperty::FontSize(30.),
        ]);
        r.add_str("Headline 😀");
        r.add_newline();
        r.add_single("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut", StyleProperty::Font("Helvetica"));

        //let text = Text::new("l 😀 i");
        //let text = Text::new("Emoji! aslkfdj salkjf 😀 👨‍👩‍👧‍👦");
        let text = Text::new(r);
        // let text = Text::new("l e");
        // let text = Text::new("World");

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
            self.transform = self.transform.scaled((delta.x / 100.) as f32);
            self.needs_composition = true;
        }

        if context.mouse.pressed_left() {
            let delta = context.mouse.position_delta;
            self.transform = self.transform.translated(-delta.x as f32, -delta.y as f32);
            self.needs_composition = true;
        }
    }

    fn compose<'a>(&mut self, composition: &mut Composition, context: &RunContext<'a>) {
        if !self.needs_composition {
            return;
        }

        let mut index = 0;

        let mut layout_context = WidgetContext {
            font_context: &mut self.font_context,
            transform: &self.transform,
            index: &mut index,
        };

        let size = Size::new(200., 300.);

        // FIXME: We get size back here, do something with it?
        self.widget.layout(&mut layout_context, size);
        // FIXME: Hand run context to widget?
        self.widget
            .compose(&mut layout_context, composition, context.elapsed);

        self.needs_composition = false;
    }
}
