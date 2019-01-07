/// RGBW color
#[derive(Clone, Default, RustcEncodable, RustcDecodable)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub w: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, w: u8) -> Self {
        Self { r, g, b, w }
    }

    /// Returns a css string of the RGB components
    pub fn rgb_to_css(&self) -> String {
        format!("rgb({}, {}, {})", self.r, self.g, self.b)
    }

    /// Returns a css string of the white component
    pub fn white_to_css(&self) -> String {
        format!("rgb({}, {}, {})", self.w, self.w, self.w)
    }
}

impl From<&Color> for [u8; 5] {
    fn from(color: &Color) -> [u8; 5] {
        [0, color.r, color.g, color.b, color.w]
    }
}
