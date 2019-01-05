/// RGBW color
#[derive(Clone, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub w: u8,
}

impl Color {
    /// Returns a css string of the RGB components
    pub fn rgb_to_css(&self) -> String {
        format!("rgb({}, {}, {})", self.r, self.g, self.b)
    }

    /// Returns a css string of the white component
    pub fn white_to_css(&self) -> String {
        format!("rgb({}, {}, {})", self.w, self.w, self.w)
    }
}
