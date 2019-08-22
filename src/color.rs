use serde_derive::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Sub};

fn lerp_component(low: f32, high: f32, percent: f32) -> f32 {
    (high - low) * percent + low
}

fn clamp_component(component: f32) -> f32 {
    if component > 1.0 {
        1.0
    } else if component < 0.0 {
        0.0
    } else {
        component
    }
}

/// RGBW color (float representation, 0.0 to 1.0)
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub w: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, w: f32) -> Self {
        Self { r, g, b, w }
    }

    pub fn clamped(&self) -> Self {
        Self {
            r: clamp_component(self.r),
            g: clamp_component(self.g),
            b: clamp_component(self.b),
            w: clamp_component(self.w),
        }
    }

    pub fn lerp(&self, other: &Self, percent: f32) -> Self {
        Self {
            r: lerp_component(self.r, other.r, percent),
            g: lerp_component(self.g, other.g, percent),
            b: lerp_component(self.b, other.b, percent),
            w: lerp_component(self.w, other.w, percent),
        }
    }

    pub fn update_clone(&mut self, reference: &Self) {
        self.r = reference.r;
        self.g = reference.g;
        self.b = reference.b;
        self.w = reference.w;
    }

    /// Returns a css string of the RGB components
    pub fn rgb_to_css(&self) -> String {
        format!(
            "rgb({}%, {}%, {}%)",
            self.r * 100.0,
            self.g * 100.0,
            self.b * 100.0
        )
    }

    /// Returns a css string of the white component
    pub fn white_to_css(&self) -> String {
        format!(
            "rgb({}%, {}%, {}%)",
            self.w * 100.0,
            self.w * 100.0,
            self.w * 100.0
        )
    }
}

impl From<&[u8; 4]> for Color {
    fn from(bytes: &[u8; 4]) -> Self {
        Self {
            r: f32::from(bytes[0]) / f32::from(<u8>::max_value()),
            g: f32::from(bytes[1]) / f32::from(<u8>::max_value()),
            b: f32::from(bytes[2]) / f32::from(<u8>::max_value()),
            w: f32::from(bytes[3]) / f32::from(<u8>::max_value()),
        }
    }
}

impl Sub for Color {
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

impl Add for Color {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
            w: self.w + other.w,
        }
    }
}

impl Div<f32> for Color {
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

impl Mul<f32> for Color {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self {
            r: self.r * scalar,
            g: self.g * scalar,
            b: self.b * scalar,
            w: self.w * scalar,
        }
    }
}
