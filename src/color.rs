use std::ops::{Div, Sub};

fn lerp_component(low: f32, high: f32, percent: f32) -> f32 {
    (high - low) * percent + low
}

/// RGBW color
#[derive(Clone, Debug, Default, PartialEq, RustcEncodable, RustcDecodable)]
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

    pub fn update_clone(&mut self, reference: &Self) {
        self.r = reference.r;
        self.g = reference.g;
        self.b = reference.b;
        self.w = reference.w;
    }

    /// Returns a css string of the RGB components
    pub fn rgb_to_css(&self) -> String {
        format!("rgb({}, {}, {})", self.r, self.g, self.b)
    }

    /// Returns a css string of the white component
    pub fn white_to_css(&self) -> String {
        format!("rgb({}, {}, {})", self.w, self.w, self.w)
    }

    /// Check if any of the (r, g, b, w) contains a particular value
    pub fn any_value<T: Into<u8>>(&self, value: T) -> bool {
        let value_into_u8 = value.into();
        value_into_u8 == self.r
            || value_into_u8 == self.g
            || value_into_u8 == self.b
            || value_into_u8 == self.w
    }
}

impl From<&Color> for [u8; 5] {
    fn from(color: &Color) -> [u8; 5] {
        [0, color.r, color.g, color.b, color.w]
    }
}

impl From<&[u8]> for Color {
    fn from(bytes: &[u8]) -> Self {
        assert!(bytes.len() > 2);
        Self {
            r: bytes[0],
            g: bytes[1],
            b: bytes[2],
            w: 0,
        }
    }
}

impl From<&FloatColor> for Color {
    fn from(color: &FloatColor) -> Self {
        Self {
            r: color.r.round() as u8,
            g: color.g.round() as u8,
            b: color.b.round() as u8,
            w: color.w.round() as u8,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FloatColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub w: f32,
}

impl FloatColor {
    pub fn lerp(&self, other: &FloatColor, percent: f32) -> FloatColor {
        Self {
            r: lerp_component(self.r, other.r, percent),
            g: lerp_component(self.g, other.g, percent),
            b: lerp_component(self.b, other.b, percent),
            w: lerp_component(self.w, other.w, percent),
        }
    }
}

impl From<&Color> for FloatColor {
    fn from(color: &Color) -> Self {
        Self {
            r: f32::from(color.r),
            g: f32::from(color.g),
            b: f32::from(color.b),
            w: f32::from(color.w),
        }
    }
}

impl Sub for FloatColor {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            r: self.r - other.r,
            g: self.g - other.g,
            b: self.b - other.b,
            w: self.w - other.w,
        }
    }
}

impl Div<f32> for FloatColor {
    type Output = Self;

    fn div(self, scalar: f32) -> Self {
        Self {
            r: self.r / scalar,
            g: self.g / scalar,
            b: self.b / scalar,
            w: self.w / scalar,
        }
    }
}
