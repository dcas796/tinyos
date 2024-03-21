use crate::vga::color::VgaColor;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct VgaPixel(pub VgaColor);
