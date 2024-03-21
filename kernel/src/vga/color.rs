use core::cmp::{max, min};
use core::ops::Add;
use micromath::F32Ext;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct VgaColor {
    red: u8,
    green: u8,
    blue: u8,
    
    hue: u16,
    saturation: u8,
    lightness: u8,
}

impl VgaColor {
    pub fn new_rgb(red: u8, green: u8, blue: u8) -> Self {
        let (hue, saturation, lightness) = Self::rgb_to_hsl(red, green, blue);
        Self { red, green, blue, hue, saturation, lightness }
    }

    pub fn new_hsl(hue: u16, saturation: u8, lightness: u8) -> Self {
        let (red, green, blue) = Self::hsl_to_rgb(hue, saturation, lightness);
        Self { red, green, blue, hue, saturation, lightness }
    }

    // TODO: Add CMYK support

    fn rgb_to_hsl(red: u8, green: u8, blue: u8) -> (u16, u8, u8) {
        let upper_m = max(max(red, green), blue) as f32;
        let lower_m = min(min(red, green), blue) as f32;
        let d = (upper_m - lower_m) / 255.0;

        let l = (upper_m + lower_m) / 510.0;
        let s = if l == 0.0 {
            0.0
        } else {
            d / (1.0 - (2.0 * l - 1.0).abs())
        };

        let red = red as f32;
        let green = green as f32;
        let blue = blue as f32;

        let mut h = (
            (red - green / 2.0 - blue / 2.0) /
                (red * red + green * green + blue * blue - red * green - red * blue - green * blue)
                    .sqrt()
        ).acos();
        if blue > green {
            h = 360.0 - h;
        }

        (h as u16, s as u8, l as u8)
    }

    fn hsl_to_rgb(hue: u16, saturation: u8, lightness: u8) -> (u8, u8, u8) {
        let saturation = saturation as f32;
        let lightness = lightness as f32;
        let d = saturation * (1.0 - (2.0 * lightness - 1.0).abs());
        let m = 255.0 * (lightness - d / 2.0);
        let x = d * (1.0 - ((hue as f32 / 60.0) % 2.0 - 1.0).abs());

        let floats = match hue {
            0..60 => (255.0 * d + m, 255.0 * x + m, m),
            60..120 => (255.0 * x + m, 255.0 * d + m, m),
            120..180 => (m, 255.0 * d + m, 255.0 * x + m),
            180..240 => (m, 255.0 * x + m, 255.0 * d + m),
            240..300 => (255.0 * x + m, m, 255.0 * d + m),
            300..360 => (255.0 * d + m, m, 255.0 * x + m),
            _ => (0.0, 0.0, 0.0)
        };

        (floats.0 as u8, floats.1 as u8, floats.2 as u8)
    }
    
    pub fn red_val(&self) -> u8 {
        self.red
    }
    
    pub fn green_val(&self) -> u8 {
        self.green
    }

    pub fn blue_val(&self) -> u8 {
        self.blue
    }

    pub fn hue(&self) -> u16 {
        self.hue
    }

    pub fn saturation(&self) -> u8 {
        self.saturation
    }

    pub fn lightness(&self) -> u8 {
        self.lightness
    }

    pub fn set_red(&mut self, red: u8) {
        self.red = red;
        (self.hue, self.saturation, self.lightness) = Self::rgb_to_hsl(self.red, self.green, self.blue);
    }

    pub fn set_green(&mut self, green: u8) {
        self.green = green;
        (self.hue, self.saturation, self.lightness) = Self::rgb_to_hsl(self.red, self.green, self.blue);
    }

    pub fn set_blue(&mut self, blue: u8) {
        self.blue = blue;
        (self.hue, self.saturation, self.lightness) = Self::rgb_to_hsl(self.red, self.green, self.blue);
    }

    pub fn set_hue(&mut self, hue: u16) {
        self.hue = hue;
        (self.red, self.green, self.blue) = Self::hsl_to_rgb(self.hue, self.saturation, self.lightness);
    }

    pub fn set_saturation(&mut self, saturation: u8) {
        self.saturation = saturation;
        (self.red, self.green, self.blue) = Self::hsl_to_rgb(self.hue, self.saturation, self.lightness);
    }

    pub fn set_lightness(&mut self, lightness: u8) {
        self.lightness = lightness;
        (self.red, self.green, self.blue) = Self::hsl_to_rgb(self.hue, self.saturation, self.lightness);
    }
}

impl Add for VgaColor {
    type Output = VgaColor;

    fn add(self, rhs: Self) -> Self::Output {
        VgaColor::new_rgb(
            self.red.saturating_add(rhs.red),
            self.green.saturating_add(rhs.green),
            self.blue.saturating_add(rhs.blue)
        )
    }
}

impl VgaColor {
    pub fn black() -> Self {
        Self::new_rgb(0, 0, 0)
    }

    pub fn dark_red() -> Self {
        Self::new_rgb(127, 0, 0)
    }

    pub fn dark_green() -> Self {
        Self::new_rgb(0, 127, 0)
    }

    pub fn dark_yellow() -> Self {
        Self::new_rgb(127, 127, 0)
    }

    pub fn dark_blue() -> Self {
        Self::new_rgb(127, 0, 0)
    }

    pub fn dark_magenta() -> Self {
        Self::new_rgb(127, 0, 127)
    }

    pub fn dark_cyan() -> Self {
        Self::new_rgb(0, 127, 127)
    }

    pub fn light_gray() -> Self {
        Self::new_rgb(192, 192, 192)
    }

    pub fn dark_gray() -> Self {
        Self::new_rgb(128, 128, 128)
    }

    pub fn red() -> Self {
        Self::new_rgb(255, 0, 0)
    }

    pub fn green() -> Self {
        Self::new_rgb(0, 255, 0)
    }

    pub fn yellow() -> Self {
        Self::new_rgb(255, 255, 0)
    }

    pub fn blue() -> Self {
        Self::new_rgb(0, 0, 255)
    }

    pub fn magenta() -> Self {
        Self::new_rgb(255, 0, 255)
    }

    pub fn cyan() -> Self {
        Self::new_rgb(0, 255, 255)
    }

    pub fn white() -> Self {
        Self::new_rgb(255, 255, 255)
    }
}
