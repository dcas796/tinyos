pub mod char;
pub mod color;
pub mod pixel;

use crate::utils::heap_array::HeapArray;
use crate::vga::char::VgaChar;
use crate::vga::color::VgaColor;
use crate::vga::pixel::VgaPixel;
use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use noto_sans_mono_bitmap::{get_raster, RasterHeight};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum VgaMode {
    Text,
    Pixels,
}

pub const TEXT_BUFFER_WIDTH: usize = 80;
pub const TEXT_BUFFER_HEIGHT: usize = 25;
pub const TEXT_BUFFER_SIZE: usize = TEXT_BUFFER_WIDTH * TEXT_BUFFER_HEIGHT * 15;

pub struct VgaScreen<'a> {
    pub mode: VgaMode,
    framebuffer: &'a mut FrameBuffer,

    text_buffer: HeapArray<VgaChar>,
    text_offset: usize,
    char_width: usize,
    char_height: usize,

    pub pixel_buffer: HeapArray<VgaPixel>,
}

impl<'a> VgaScreen<'a> {
    pub fn new(framebuffer: &'a mut FrameBuffer) -> Self {
        let info = framebuffer.info();
        assert_eq!(info.pixel_format, PixelFormat::Bgr);
        assert_eq!(info.bytes_per_pixel, 3);
        let char_width = info.width / TEXT_BUFFER_WIDTH;
        let char_height = info.height / TEXT_BUFFER_HEIGHT;
        let pixel_buffer_length = info.byte_len;
        Self {
            mode: VgaMode::Text,
            framebuffer,
            text_buffer: HeapArray::new(TEXT_BUFFER_SIZE).unwrap(),
            text_offset: 0,
            char_width,
            char_height,
            pixel_buffer: HeapArray::new(pixel_buffer_length / 3).unwrap(),
        }
    }

    pub fn clear_screen(&mut self) {
        self.text_buffer.fill(VgaChar::default());
        self.pixel_buffer.fill(VgaPixel(VgaColor::black()));
        self.draw();
    }

    pub fn draw(&mut self) {
        match self.mode {
            VgaMode::Text => self.draw_text(),
            VgaMode::Pixels => self.draw_pixels(),
        }
    }

    fn draw_text(&mut self) {
        let starting_row = self.text_offset / self.char_height;
        for y in 0..(TEXT_BUFFER_HEIGHT + 1) {
            for x in 0..TEXT_BUFFER_WIDTH {
                let char = self.text_buffer[(y + starting_row) * TEXT_BUFFER_WIDTH + x];
                self.draw_char(&char, x, y + (self.text_offset % self.char_height));
            }
        }
    }

    fn draw_char(&mut self, char: &VgaChar, x: usize, y: usize) {
        let raster = get_raster(char.char, char.style.weight, RasterHeight::Size16)
            .unwrap_or(get_raster(' ', char.style.weight, RasterHeight::Size16).unwrap());

        let pixel_x = x * self.char_width;
        let pixel_y = y * self.char_height;
        for (y, row) in raster.raster().iter().enumerate() {
            for (x, lightness) in row.iter().enumerate() {
                let mut fore_color = char.style.foreground.clone();
                fore_color.set_lightness(*lightness);
                let mut back_color = char.style.background.clone();
                back_color.set_lightness(255 - lightness);
                self.buffer_set(pixel_x + x, pixel_y + y, VgaPixel(fore_color + back_color));
            }
        }
    }

    fn draw_pixels(&mut self) {
        for (i, pixel) in self.pixel_buffer.clone().iter().enumerate() {
            self.buffer_set(
                i % self.framebuffer.info().width,
                i / self.framebuffer.info().width,
                *pixel,
            )
        }
    }

    fn buffer(&self) -> &[u8] {
        self.framebuffer.buffer()
    }

    fn buffer_mut(&mut self) -> &mut [u8] {
        self.framebuffer.buffer_mut()
    }

    fn buffer_info(&self) -> FrameBufferInfo {
        self.framebuffer.info()
    }

    fn buffer_pos(&self, x: usize, y: usize) -> usize {
        (y * self.buffer_info().stride + x) * 3
    }

    fn buffer_get(&self, x: usize, y: usize) -> VgaPixel {
        let pos = self.buffer_pos(x, y);
        let buffer = self.buffer();
        let r = buffer[pos + 2];
        let g = buffer[pos + 1];
        let b = buffer[pos];
        VgaPixel(VgaColor::new_rgb(r, g, b))
    }

    fn buffer_set(&mut self, x: usize, y: usize, pixel: VgaPixel) {
        let pos = self.buffer_pos(x, y);
        let buffer = self.buffer_mut();
        buffer[pos + 2] = pixel.0.red_val();
        buffer[pos + 1] = pixel.0.green_val();
        buffer[pos] = pixel.0.blue_val();
    }
}
