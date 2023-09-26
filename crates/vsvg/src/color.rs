use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const DARK_GRAY: Self = Self::rgb(96, 96, 96);
    pub const GRAY: Self = Self::rgb(160, 160, 160);
    pub const LIGHT_GRAY: Self = Self::rgb(220, 220, 220);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const BROWN: Self = Self::rgb(165, 42, 42);
    pub const DARK_RED: Self = Self::rgb(0x8B, 0, 0);
    pub const RED: Self = Self::rgb(255, 0, 0);
    pub const LIGHT_RED: Self = Self::rgb(255, 128, 128);
    pub const YELLOW: Self = Self::rgb(255, 255, 0);
    pub const LIGHT_YELLOW: Self = Self::rgb(255, 255, 0xE0);
    pub const KHAKI: Self = Self::rgb(240, 230, 140);
    pub const DARK_GREEN: Self = Self::rgb(0, 0x64, 0);
    pub const GREEN: Self = Self::rgb(0, 255, 0);
    pub const LIGHT_GREEN: Self = Self::rgb(0x90, 0xEE, 0x90);
    pub const DARK_BLUE: Self = Self::rgb(0, 0, 0x8B);
    pub const BLUE: Self = Self::rgb(0, 0, 255);
    pub const LIGHT_BLUE: Self = Self::rgb(0xAD, 0xD8, 0xE6);
    pub const GOLD: Self = Self::rgb(255, 215, 0);

    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 0xFF }
    }

    #[must_use]
    pub const fn gray(v: u8) -> Self {
        Self {
            r: v,
            g: v,
            b: v,
            a: 0xFF,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    #[must_use]
    pub fn with_opacity(&self, opacity: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: (opacity * 255.0) as u8,
        }
    }

    #[must_use]
    pub const fn to_rgba(&self) -> u32 {
        self.r as u32 | (self.g as u32) << 8 | (self.b as u32) << 16 | (self.a as u32) << 24
    }

    #[must_use]
    pub fn to_rgb_string(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    #[must_use]
    pub fn opacity(&self) -> f32 {
        f32::from(self.a) / 255.0
    }
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#{:02x}{:02x}{:02x}{:02x}",
            self.r, self.g, self.b, self.a
        )
    }
}
