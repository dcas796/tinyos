use noto_sans_mono_bitmap::FontWeight;
use crate::vga::color::VgaColor;

#[derive(Debug, Copy, Clone)]
pub struct VgaStyle {
    pub background: VgaColor,
    pub foreground: VgaColor,
    pub weight: FontWeight,
}

impl VgaStyle {
    pub fn new(background: VgaColor, foreground: VgaColor, weight: FontWeight) -> Self {
        Self { background, foreground, weight }
    }
}

impl PartialEq for VgaStyle {
    fn eq(&self, other: &Self) -> bool {
        self.background == other.background &&
            self.foreground == other.foreground &&
            self.weight.val() == other.weight.val()
    }
}

impl Eq for VgaStyle {}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct VgaChar {
    pub char: char,
    pub style: VgaStyle,
}

impl VgaChar {
    pub fn new(char: char, style: VgaStyle) -> Self {
        Self { char, style }
    }

    pub fn default() -> Self {
        Self::new(' ', VgaStyle::new(
            VgaColor::black(), VgaColor::black(), FontWeight::Regular)
        )
    }
}