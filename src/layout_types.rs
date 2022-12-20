use forma::prelude::*;
use parley::{style::Brush, FontContext};

use crate::types::Size;

use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormaBrush {
    pub fill: Fill,
}

impl Default for FormaBrush {
    fn default() -> Self {
        Self {
            fill: Fill::Solid(Color {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 1.,
            }),
        }
    }
}

impl Brush for FormaBrush {}

pub trait Widget {
    fn layout<'a>(&mut self, ctx: &mut WidgetContext<'a>, proposed_size: Size) -> Size;
    fn compose<'a>(
        &mut self,
        ctx: &WidgetContext<'a>,
        composition: &mut Composition,
        elapsed: Duration,
    );
}

pub struct WidgetContext<'a> {
    pub font_context: &'a mut FontContext,
    pub transform: &'a AffineTransform,
    pub index: &'a mut u32,
    pub clip: bool,
}

/// Key for building a glyph cache
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CacheKey {
    /// Font ID
    pub font_id: usize,
    /// Glyph ID
    pub glyph_id: u16,
    /// Font size in pixels
    pub font_size: i32,
}
