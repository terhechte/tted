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
    fn measure(&mut self) -> (Size, Size);
    fn layout<'a>(&mut self, ctx: &mut LayoutContext<'a>, proposed_size: Size) -> Size;
    fn compose<'a>(
        &mut self,
        ctx: &LayoutContext<'a>,
        composition: &mut Composition,
        elapsed: Duration,
    );
}

pub struct LayoutContext<'a> {
    pub font_context: &'a mut FontContext,
    pub transform: &'a AffineTransform,
    pub index: u32,
}
