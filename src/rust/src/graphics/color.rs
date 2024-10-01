pub struct Color(i32);

impl Color {
    /// Convert to an R color.
    pub fn to_i32(&self) -> i32 {
        self.0
    }

    /// Generate a color from a CSS-like hex number.
    /// eg. `Color::hex(0xF0F8FF)`
    pub fn hex(hex: u32) -> Color {
        let red = (hex >> 16) & 0xff;
        let green = (hex >> 8) & 0xff;
        let blue = hex & 0xff;
        Color(red as i32 | (green as i32) << 8 | (blue as i32) << 16 | 0xff << 24)
    }

    /// Generate a color from a 3 digit CSS-like hex number.
    /// eg. `Color::hex(0xF0F)`
    pub fn hex3(hex: u32) -> Color {
        let red = ((hex >> 8) & 0xf) * 0xff / 0x0f;
        let green = ((hex >> 4) & 0xf) * 0xff / 0x0f;
        let blue = (hex & 0xf) * 0xff / 0x0f;
        Color(red as i32 | (green as i32) << 8 | (blue as i32) << 16 | 0xff << 24)
    }

    /// Generate a color from rgb components (0-255).
    pub fn rgb(red: u8, green: u8, blue: u8) -> Color {
        Color(red as i32 | (green as i32) << 8 | (blue as i32) << 16 | 0xff << 24)
    }

    /// Generate a color from rgba components (0-255).
    pub fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Color {
        Color(red as i32 | (green as i32) << 8 | (blue as i32) << 16 | (alpha as i32) << 24)
    }
}
