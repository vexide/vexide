pub trait IntoRgb {
    fn into_rgb(self) -> Rgb;
}

impl<T: Into<u32>> IntoRgb for T {
    fn into_rgb(self: T) -> Rgb {
        self.into() // u32
            .into() // Rgb
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const WHITE: Self = Self::new(0xFF, 0xFF, 0xFF);
    pub const SILVER: Self = Self::new(0xC0, 0xC0, 0xC0);
    pub const GRAY: Self = Self::new(0x80, 0x80, 0x80);
    pub const BLACK: Self = Self::new(0x00, 0x00, 0x00);
    pub const RED: Self = Self::new(0xFF, 0x00, 0x00);
    pub const MAROON: Self = Self::new(0x80, 0x00, 0x00);
    pub const YELLOW: Self = Self::new(0xFF, 0xFF, 0x00);
    pub const OLIVE: Self = Self::new(0x80, 0x80, 0x00);
    pub const LIME: Self = Self::new(0x00, 0xFF, 0x00);
    pub const GREEN: Self = Self::new(0x00, 0x80, 0x00);
    pub const CYAN: Self = Self::new(0x00, 0xFF, 0xFF);
    pub const AQUA: Self = Self::CYAN;
    pub const TEAL: Self = Self::new(0x00, 0x80, 0x80);
    pub const BLUE: Self = Self::new(0x00, 0x00, 0xFF);
    pub const NAVY: Self = Self::new(0x00, 0x00, 0x80);
    pub const MAGENTA: Self = Self::new(0xFF, 0x00, 0xFF);
    pub const PURPLE: Self = Self::new(0x80, 0x00, 0x80);
    pub const ORANGE: Self = Self::new(0xFF, 0xA5, 0x00);

    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self {
            r: red,
            g: green,
            b: blue,
        }
    }

    pub fn red(&self) -> u8 {
        self.r
    }

    pub fn green(&self) -> u8 {
        self.g
    }

    pub fn blue(&self) -> u8 {
        self.b
    }
}

impl From<(u8, u8, u8)> for Rgb {
    fn from(tuple: (u8, u8, u8)) -> Self {
        Self {
            r: tuple.0,
            g: tuple.1,
            b: tuple.2,
        }
    }
}

impl From<Rgb> for (u8, u8, u8) {
    fn from(value: Rgb) -> (u8, u8, u8) {
        (value.r, value.g, value.b)
    }
}

impl From<Rgb> for u32 {
    fn from(value: Rgb) -> u32 {
        ((value.r as u32) << 16) + ((value.g as u32) << 8) + value.b as u32
    }
}

const BITMASK: u32 = 0b11111111;
impl From<u32> for Rgb {
    fn from(value: u32) -> Self {
        Self {
            r: ((value >> 16) & BITMASK) as _,
            g: ((value >> 8) & BITMASK) as _,
            b: (value & BITMASK) as _,
        }
    }
}
