use crate::layout_types::FormaBrush;
use parley::style::StyleProperty as ParleyStyleProperty;
use parley::style::{FontFamily, FontStack};
use parley::{FontContext, Layout, LayoutContext};
use std::ops::Range;

/// Simplification over `parley::style::StyleProperty` to
/// build Rich Text in a simpler manner. Less performant.
#[derive(Debug)]
pub struct RichText {
    defaults: Vec<StyleProperty>,
    stack: Vec<(Range<usize>, StyleProperty)>,
    text: String,
}

impl RichText {
    pub fn new<const N: usize>(defaults: [StyleProperty; N]) -> Self {
        RichText {
            defaults: defaults.to_vec(),
            stack: Vec::with_capacity(64),
            text: String::new(),
        }
    }

    pub fn attribute_count(&self) -> usize {
        self.stack.len()
    }

    pub fn slice(&self, range: Range<usize>) -> &str {
        &self.text[range]
    }

    pub fn add_str<'a>(&mut self, text: impl Into<&'a str>) {
        self.text.push_str(text.into());
    }

    pub fn add_single<'a>(&mut self, text: impl Into<&'a str>, property: StyleProperty) {
        let len = self.text.len();
        let text = text.into();
        let range = len..(len + text.len());
        self.text.push_str(text);
        self.stack.push((range, property))
    }

    pub fn add_many<'a, const S: usize>(
        &mut self,
        text: impl Into<&'a str>,
        properties: [StyleProperty; S],
    ) {
        let len = self.text.len();
        let text = text.into();
        let range = len..(len + text.len());
        self.text.push_str(text);
        for item in properties {
            self.stack.push((range.clone(), item))
        }
    }

    pub fn add_newline(&mut self) {
        self.text.push('\n');
    }
}

#[derive(Debug, Clone)]
pub enum StyleProperty {
    Font(&'static str),
    FontSize(f32),
    FontStyle(parley::style::FontStyle),
    FontWeight(parley::style::FontWeight),
    Brush(FormaBrush),
    Underline(bool),
    LineHeight(f32),
    LetterSpacing(f32),
}

impl StyleProperty {
    fn as_parley<'a>(&self) -> ParleyStyleProperty<'a, FormaBrush> {
        use ParleyStyleProperty as Py;
        match self {
            StyleProperty::Font(font) => Py::FontStack(FontStack::Single(FontFamily::Named(font))),
            StyleProperty::FontSize(size) => Py::FontSize(*size),
            StyleProperty::FontStyle(style) => Py::FontStyle(*style),
            StyleProperty::FontWeight(weight) => Py::FontWeight(*weight),
            StyleProperty::Brush(brush) => Py::Brush(brush.clone()),
            StyleProperty::Underline(underline) => Py::Underline(*underline),
            StyleProperty::LineHeight(line_height) => Py::LineHeight(*line_height),
            StyleProperty::LetterSpacing(spacing) => Py::LetterSpacing(*spacing),
        }
    }
}

impl RichText {
    pub fn build(
        &self,
        layout_context: &mut LayoutContext<FormaBrush>,
        font_context: &mut FontContext,
    ) -> Layout<FormaBrush> {
        let mut layout_builder = layout_context.ranged_builder(font_context, &self.text, 1.0);
        for property in self.defaults.iter() {
            layout_builder.push_default(&property.as_parley());
        }
        for (range, property) in self.stack.iter() {
            layout_builder.push(&property.as_parley(), range.clone());
        }
        layout_builder.build()
    }
}
