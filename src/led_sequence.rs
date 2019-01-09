use std::collections::VecDeque;
use std::fs::File;
use std::path::Path;

use crate::color::{Color, FloatColor};

// 60 "frames" per second for smoothness
const RESULUTION: f32 = 30.0;
const DURATION: f32 = 0.5;

#[derive(Debug)]
enum LedSequenceType {
    Color,
    Gradient,
}

#[derive(Debug)]
struct LedSequenceInfo {
    pub sequence_type: LedSequenceType,
    pub name: String,
    pub duration: f32,
    pub repeat: bool,
}

#[derive(Debug)]
pub struct LedSequence {
    pub colors: VecDeque<Color>,
    pub delays: VecDeque<f32>,
}

impl LedSequence {
    /// Linearly interpolate between two colors, for the default duration and
    /// resolution
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

    /// Load a gradient or single colour from a png file
    ///
    /// Make the images 1024 x 100 px; the easiest is with GIMP.
    ///
    /// Name the files according to this convention:
    ///
    /// ```
    /// <color/gradient>_<name>_<duration?>_<repeat?>.png
    /// ```
    ///
    /// Examples:
    ///
    /// ```
    /// color_red.png
    ///
    /// gradient_sunrise_600.png
    ///
    /// gradient_rainbow_20_repeat.png
    /// ```
    pub fn from_png(fade_from: &Color, img_path: &Path) -> Self {
        let name = img_path
            .file_stem()
            .expect("Unable to extract file name")
            .to_str()
            .expect("Invalid_filename")
            .to_string();

        let tokens: Vec<_> = name.split('_').collect();
        assert!(tokens.len() > 1 && tokens.len() < 5);

        let info = if tokens[0] == "color" {
            assert_eq!(tokens.len(), 2);
            LedSequenceInfo {
                sequence_type: LedSequenceType::Color,
                name: tokens[1].to_string(),
                duration: 0.0,
                repeat: false,
            }
        } else if tokens[0] == "gradient" {
            assert!(tokens.len() > 2);
            let duration = tokens[2]
                .parse::<f32>()
                .expect("Unable to parse gradient duration");
            LedSequenceInfo {
                sequence_type: LedSequenceType::Gradient,
                name: tokens[1].to_string(),
                duration,
                repeat: tokens.contains(&"repeat"),
            }
        } else {
            unreachable!("Incorrect first token in png file name");
        };

        let decoder = png::Decoder::new(File::open(img_path).unwrap());
        let (png_info, mut reader) = decoder.read_info().unwrap();
        let mut buf = vec![0; png_info.buffer_size()];
        reader.next_frame(&mut buf).unwrap();

        let first_white_index = 3
            * png_info.width as usize
            * (png_info.height as usize / 2) as usize;
        let first_color =
            Color::new(buf[0], buf[1], buf[2], buf[first_white_index]);

        let initial_fade = Self::from_color_lerp(fade_from, &first_color);

        match info.sequence_type {
            LedSequenceType::Color => initial_fade,
            LedSequenceType::Gradient => {
                let mut colors =
                    VecDeque::with_capacity(png_info.width as usize);
                let mut delays =
                    VecDeque::with_capacity(png_info.width as usize);
                let delay = (3.0 * info.duration) / png_info.width as f32;
                for i in (0..(png_info.width as usize * 3)).step_by(3) {
                    let color_i = Color::new(
                        buf[i],
                        buf[i + 1],
                        buf[i + 2],
                        buf[i + first_white_index],
                    );
                    colors.push_back(color_i);
                    delays.push_back(delay);
                }
                let sequence = Self { colors, delays };
                initial_fade.chain(sequence)
            }
        }
    }

    /// Chain two LED sequences together, consuming both
    fn chain(mut self, other: LedSequence) -> Self {
        self.colors.extend(other.colors);
        self.delays.extend(other.delays);
        Self {
            colors: self.colors,
            delays: self.delays,
        }
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
