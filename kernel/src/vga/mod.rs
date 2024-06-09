pub mod char;
pub mod color;
pub mod pixel;

use crate::utils::heap_array::HeapArray;
use crate::vga::char::VgaChar;
use crate::vga::color::VgaColor;
use crate::vga::pixel::VgaPixel;
use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
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
    text_offset: usize,
    pixel_buffer: HeapArray<VgaPixel>,
}

impl<'a> VgaScreen<'a> {
    pub fn new(framebuffer: &'a mut FrameBuffer) -> Result<Self, VgaError> {
        let info = framebuffer.info();
        // TODO: Support more pixel formats
        assert_eq!(info.pixel_format, PixelFormat::Bgr);
        assert_eq!(info.bytes_per_pixel, 3);
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
            VgaMode::Text => self.draw_text(),
            VgaMode::Pixels => self.draw_pixels(),
        }
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

    fn draw_text(&mut self) {
        let starting_row = self.text_offset / CHAR_HEIGHT;
        for y in 0..(TEXT_SCREEN_ROWS + 1) {
            for x in 0..TEXT_SCREEN_COLS {
                let char = self.text_buffer[(y + starting_row) * TEXT_SCREEN_COLS + x];
                self.draw_char(
                    &char,
                    x * CHAR_WIDTH,
                    y * CHAR_HEIGHT + (self.text_offset % CHAR_HEIGHT),
                );
            }
        }
    }

    fn draw_char(&mut self, char: &VgaChar, x: usize, y: usize) {
        let raster = get_raster(char.char, CHAR_WEIGHT, CHAR_SIZE)
            .unwrap_or(
                get_raster(' ', CHAR_WEIGHT, CHAR_SIZE)
                    .expect("Cannot get default raster for char while drawing to screen."),
            )
            .raster();

        for (i, row) in raster.iter().enumerate() {
            for (j, lightness) in row.iter().enumerate() {
                // TODO: Actually display the foreground and background colors set by VgaChar
                self.buffer_set(
                    x + j,
                    y + i,
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
        let pos = self.buffer_pos(x, y);
        if (pos + 2) >= self.buffer().len() {
            // Don't draw the pixel to the screen
            return;
        }
        let buffer = self.buffer_mut();
        buffer[pos + 2] = pixel.0.red_val();
        buffer[pos + 1] = pixel.0.green_val();
        buffer[pos] = pixel.0.blue_val();
    }
}
