pub mod char;
pub mod color;
pub mod pixel;

use crate::utils::heap_array::HeapArray;
use crate::vga::char::VgaChar;
use crate::vga::color::VgaColor;
use crate::vga::pixel::VgaPixel;
use alloc::borrow::Cow;
use alloc::vec::Vec;
use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use char::VgaStyle;
use noto_sans_mono_bitmap::{get_raster, get_raster_width, FontWeight, RasterHeight};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VgaError {
    HeapArrayError(crate::utils::heap_array::HeapArrayError),
}

impl From<crate::utils::heap_array::HeapArrayError> for VgaError {
    fn from(value: crate::utils::heap_array::HeapArrayError) -> Self {
        Self::HeapArrayError(value)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum VgaMode {
    Text,
    Pixels,
}

pub const TEXT_SCREEN_COLS: usize = 142;
pub const TEXT_SCREEN_ROWS: usize = 45;
pub const TEXT_BUFFER_SIZE: usize = TEXT_SCREEN_COLS * TEXT_SCREEN_ROWS * 15;

pub const CHAR_WEIGHT: FontWeight = FontWeight::Regular;
pub const CHAR_SIZE: RasterHeight = RasterHeight::Size16;
pub const CHAR_WIDTH: usize = get_raster_width(CHAR_WEIGHT, CHAR_SIZE);
pub const CHAR_HEIGHT: usize = CHAR_SIZE.val();

pub struct VgaScreen<'a> {
    pub mode: VgaMode,
    framebuffer: &'a mut FrameBuffer,

    text_buffer: HeapArray<VgaChar>,
    pub text_offset: usize,
    pixel_buffer: HeapArray<VgaPixel>,
}

impl<'a> VgaScreen<'a> {
    pub fn new(framebuffer: &'a mut FrameBuffer) -> Result<Self, VgaError> {
        let info = framebuffer.info();
        // TODO: Support more pixel formats
        assert_eq!(
            info.pixel_format,
            PixelFormat::Bgr,
            "{:?} format not supported.",
            info.pixel_format
        );
        assert_eq!(
            info.bytes_per_pixel, 3,
            "{} bytes per pixel not supported.",
            info.bytes_per_pixel
        );
        let pixel_buffer_length = info.byte_len;
        let text_buffer = HeapArray::new(TEXT_BUFFER_SIZE)?;
        let pixel_buffer = HeapArray::new(pixel_buffer_length / 3)?;
        let mut screen = Self {
            mode: VgaMode::Text,
            framebuffer,
            text_buffer,
            text_offset: 0,
            pixel_buffer,
        };
        screen.clear_buffers();
        Ok(screen)
    }

    pub fn text_buffer(&self) -> &HeapArray<VgaChar> {
        &self.text_buffer
    }

    pub fn text_buffer_mut(&mut self) -> &mut HeapArray<VgaChar> {
        &mut self.text_buffer
    }

    pub fn pixel_buffer(&self) -> &HeapArray<VgaPixel> {
        &self.pixel_buffer
    }

    pub fn pixel_buffer_mut(&mut self) -> &mut HeapArray<VgaPixel> {
        &mut self.pixel_buffer
    }

    pub fn clear_buffers(&mut self) {
        self.text_buffer.fill(VgaChar::default());
        self.pixel_buffer.fill(VgaPixel(VgaColor::black()));
    }

    pub fn clear_screen(&mut self) {
        self.clear_buffers();
        self.buffer_mut().fill(0);
    }

    pub fn draw(&mut self) {
        match self.mode {
            VgaMode::Text => self.draw_text_buffer(),
            VgaMode::Pixels => self.draw_pixels(),
        }
    }

    pub fn print_text(&mut self, col: usize, row: usize, text: &str, style: VgaStyle) {
        // Update the text buffer with the new string
        let mut text = Cow::from(text);
        let text_buffer_index = row * TEXT_SCREEN_COLS + col;

        if text_buffer_index + text.len() > self.text_buffer.len() {
            text.to_mut()
                .truncate(self.text_buffer.len() - text_buffer_index);
        }

        for (i, char) in text.chars().enumerate() {
            self.text_buffer_mut()[text_buffer_index + i] = VgaChar::new(char, style);
        }

        // Draw the text onto the screen
        let chars: Vec<VgaChar> = text
            .as_ref()
            .chars()
            .map(|char| VgaChar::new(char, style))
            .collect();
        self.draw_chars(col, row, &chars);
    }

    fn draw_text_buffer(&mut self) {
        let chars = self.text_buffer.clone();
        self.draw_chars(0, 0, &chars);
    }

    fn draw_chars(&mut self, col: usize, row: usize, chars: &[VgaChar]) {
        let mut curr_col = col;
        for char in chars {
            self.draw_char(
                char,
                (curr_col * CHAR_WIDTH) as isize,
                (row * CHAR_HEIGHT) as isize - self.text_offset as isize,
            );
            curr_col += 1;
        }
    }

    fn draw_char(&mut self, char: &VgaChar, x: isize, y: isize) {
        let raster = get_raster(char.char, CHAR_WEIGHT, CHAR_SIZE)
            .unwrap_or(
                get_raster(' ', CHAR_WEIGHT, CHAR_SIZE)
                    .expect("Cannot get default raster for char while drawing to screen."),
            )
            .raster();

        for (i, row) in raster.iter().enumerate() {
            for (j, lightness) in row.iter().enumerate() {
                if (x + j as isize) < 0 || (y + i as isize) < 0 {
                    continue;
                }

                // TODO: Actually display the foreground and background colors set by VgaChar
                self.buffer_set(
                    (x + j as isize) as usize,
                    (y + i as isize) as usize,
                    VgaPixel(VgaColor::new_rgb(*lightness, *lightness, *lightness)),
                )
            }
        }
    }

    fn draw_pixels(&mut self) {
        // WTF is this???
        for (i, pixel) in self.pixel_buffer.clone().iter().enumerate() {
            self.buffer_set(
                i % self.buffer_info().width,
                i / self.buffer_info().width,
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
        if x >= self.buffer_info().width || y >= self.buffer_info().height {
            return;
        }
        let pos = self.buffer_pos(x, y);
        let buffer = self.buffer_mut();
        buffer[pos + 2] = pixel.0.red_val();
        buffer[pos + 1] = pixel.0.green_val();
        buffer[pos] = pixel.0.blue_val();
    }
}
