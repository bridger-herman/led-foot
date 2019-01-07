use std::collections::VecDeque;

use crate::color::{Color, FloatColor};

// 60 "frames" per second for smoothness
const RESULUTION: f32 = 30.0;
const DURATION: f32 = 0.5;

#[derive(Debug)]
pub struct LedSequence {
    pub colors: VecDeque<Color>,
    pub delays: VecDeque<f32>,
}

impl LedSequence {
    pub fn from_color_lerp(start_color: &Color, end_color: &Color) -> Self {
        let (start_color, end_color) = (
            <FloatColor>::from(start_color),
            <FloatColor>::from(end_color),
        );

        let mut colors = VecDeque::with_capacity(RESULUTION as usize);
        let mut delays = VecDeque::with_capacity(RESULUTION as usize);

        for i in 0..RESULUTION as usize {
            let percent = i as f32 / RESULUTION;
            delays.push_back(DURATION / RESULUTION);
            colors.push_back(<Color>::from(
                &start_color.lerp(&end_color, percent),
            ));
        }

        Self { colors, delays }
    }
}

impl Iterator for &mut LedSequence {
    type Item = (f32, Color);

    fn next(&mut self) -> Option<Self::Item> {
        let (delay, color) = (self.delays.pop_front(), self.colors.pop_front());
        if delay.is_none() || color.is_none() {
            None
        } else {
            Some((delay.unwrap(), color.unwrap()))
        }
    }
}
