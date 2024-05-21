use std::collections::VecDeque;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde_derive::{Deserialize, Serialize};

use crate::color::Color;

/// 30 "frames" per second for smoothness
pub const RESOLUTION: f32 = 30.0;

/// How long the initial fade between sequences should be
pub const FADE_DURATION: f32 = 0.5;

/// Median filter size for initial
const MEDIAN_FILTER_SIZE: usize = 51;

/// Folder where all LED sequences are located
pub const SEQUENCE_PATH: &str = "led-foot-sequences";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LedSequenceType {
    Color,
    Gradient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedSequenceInfo {
    pub sequence_type: LedSequenceType,
    pub name: String,
    pub duration: f32,
    pub repeat: bool,
}

#[derive(Debug, Deserialize)]
pub struct LedColorPoints {
    pub color_points: Vec<Color>,
    pub percent_points: Vec<f32>,
    pub info: LedSequenceInfo,
}

#[derive(Debug, Clone)]
pub struct LedSequence {
    pub colors: VecDeque<Color>,
    pub info: LedSequenceInfo,
    index: usize,
    repeat_start: usize,
}

impl LedSequence {
    /// Linearly interpolate between two colors, for the default duration and
    /// resolution
    pub fn from_color_lerp(start_color: &Color, end_color: &Color) -> Self {
        let mut colors = VecDeque::with_capacity(RESOLUTION as usize);

        for i in 0..=(RESOLUTION as usize) {
            let percent = i as f32 / RESOLUTION;
            colors.push_back(start_color.lerp(&end_color, percent));
        }

        let mut sequence = Self::default();
        sequence.colors = colors;
        sequence.info = LedSequenceInfo {
            sequence_type: LedSequenceType::Color,
            name: "lerp".to_string(),
            duration: FADE_DURATION,
            repeat: false,
        };

        sequence
    }

    /// Fade from a start color to black, over a duration
    pub fn fade_to_black(start_color: &Color, duration: f32) -> Self {
        let num_elements = (duration * RESOLUTION) as usize;
        let end_color = Color::new(0.0, 0.0, 0.0, 0.0);
        let mut colors = VecDeque::with_capacity(num_elements);

        for i in 0..=(num_elements) {
            let percent = i as f32 / (duration * RESOLUTION);
            // Adjust the next color's white value to fade quicker to black
            let mut next_color = start_color.lerp(&end_color, percent);
            next_color.w = start_color.w - start_color.w * percent.cbrt();
            colors.push_back(next_color);
        }

        let mut sequence = Self::default();
        sequence.colors = colors;
        sequence.info = LedSequenceInfo {
            sequence_type: LedSequenceType::Color,
            name: "fade-to-black".to_string(),
            duration,
            repeat: false,
        };

        sequence
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

        let decoder =
            png::Decoder::new(File::open(img_path).unwrap_or_else(|_| {
                panic!("Unable to open sequence file {:?}", img_path)
            }));

        let mut reader = decoder.read_info().expect("Unable to decode png");
        let mut buf = vec![0; reader.output_buffer_size()];
        reader.next_frame(&mut buf).unwrap();

        let width = reader.info().width;
        let height = reader.info().height;

        let first_white_index =
            3 * width as usize * (height as usize / 2) as usize;

        match info.sequence_type {
            LedSequenceType::Color => {
                let raw_color =
                    [buf[0], buf[1], buf[2], buf[first_white_index]];
                let first_color = Color::from(&raw_color);
                Self::from_color_lerp(fade_from, &first_color)
            }
            LedSequenceType::Gradient => {
                let mut colors = VecDeque::with_capacity(width as usize);
                for i in (0..(width as usize * 3)).step_by(3) {
                    let raw_color = [
                        buf[i],
                        buf[i + 1],
                        buf[i + 2],
                        buf[i + first_white_index],
                    ];
                    let color_i = Color::from(&raw_color);
                    colors.push_back(color_i);
                }

                let sequence = Self {
                    colors,
                    info,
                    index: 0,
                    repeat_start: 0,
                }
                .smooth_colors()
                .resample();

                let initial_fade =
                    Self::from_color_lerp(fade_from, &sequence.colors[0]);
                let fade_len = initial_fade.colors.len();

                initial_fade.chain(sequence).with_repeat_start(fade_len)
            }
        }
    }

    pub fn from_color_points(
        fade_from: &Color,
        points_path: &Path,
    ) -> Result<Self, std::io::Error> {
        let mut file = File::open(points_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let points: LedColorPoints = serde_json::from_str(&contents)?;

        match points.info.sequence_type {
            LedSequenceType::Color => {
                Ok(Self::from_color_lerp(fade_from, &points.color_points[0]))
            }
            LedSequenceType::Gradient => {
                let num_samples = (RESOLUTION * points.info.duration) as usize;

                let mut colors = VecDeque::with_capacity(num_samples);
                let mut color_index = 0;
                for sample_index in 0..=num_samples {
                    let overall_percent =
                        sample_index as f32 / num_samples as f32;
                    let lerp_percent = 1.0
                        - ((points.percent_points[color_index + 1]
                            - overall_percent)
                            / (points.percent_points[color_index + 1]
                                - points.percent_points[color_index]));

                    colors.push_back(points.color_points[color_index].lerp(
                        &points.color_points[color_index + 1],
                        lerp_percent,
                    ));

                    if overall_percent >= points.percent_points[color_index + 1]
                    {
                        color_index += 1;
                    }
                }

                let mut sequence = Self::default();
                sequence.info = points.info;
                sequence.colors = colors;

                let initial_fade =
                    Self::from_color_lerp(fade_from, &sequence.colors[0]);
                let fade_len = initial_fade.colors.len();

                Ok(initial_fade.chain(sequence).with_repeat_start(fade_len))
            }
        }
    }

    /// Sets the index that the iterator loops back to
    pub fn with_repeat_start(mut self, repeat_start: usize) -> Self {
        self.repeat_start = repeat_start;
        self
    }

    /// Chain two LED sequences together, consuming both
    fn chain(mut self, other: LedSequence) -> Self {
        self.colors.extend(other.colors);
        self.info = {
            let mut inf = other.info.clone();
            inf.duration = self.info.duration + other.info.duration;
            inf
        };
        self
    }

    /// Downsample (or upsample) the gradient so it is smooth (say, 30 frames
    /// per second)
    ///
    /// Uses a tent filter to obtain a resampled gradient
    fn resample(mut self) -> Self {
        let num_samples = RESOLUTION * self.info.duration;
        let filter_size =
            (self.colors.len() as f32 / num_samples.round()).round() as isize;

        // Hack to allow upsampling
        let filter_size = if filter_size < 3 { 3 } else { filter_size };

        let mut new_colors = VecDeque::with_capacity(num_samples as usize);

        for i in 0..(num_samples as usize) {
            let percent = i as f32 / num_samples;
            let center_index = (percent * (self.colors.len() as f32)) as isize;

            let mut sum = Color::default();
            let mut counted = 0;
            for filter_index in (-filter_size / 2)..(filter_size / 2) {
                // Absolute value function to mimic tent
                let tent_value = 1.0
                    - ((filter_index * 2) as f32 / filter_size as f32).abs();

                let png_index = filter_index + center_index;
                if png_index >= 0 && png_index < self.colors.len() as isize {
                    sum = sum
                        + self.colors[png_index as usize].clone() * tent_value;
                    counted += 1;
                }
            }
            let avg = sum / (counted as f32 / 2.0);
            new_colors.push_back(avg.clamped());
        }
        self.colors = new_colors;
        self
    }

    /// Use a median filter to eliminate noise, consuming self
    ///
    /// Useful for gradients from png images, which tend to have noise
    /// Also cuts out values <= 1 and makes them 0 to avoid color stuttering
    fn smooth_colors(mut self) -> Self {
        let mut r_filter = median::Filter::new(MEDIAN_FILTER_SIZE);
        let mut g_filter = median::Filter::new(MEDIAN_FILTER_SIZE);
        let mut b_filter = median::Filter::new(MEDIAN_FILTER_SIZE);
        let mut w_filter = median::Filter::new(MEDIAN_FILTER_SIZE);

        let mut new_colors = VecDeque::with_capacity(self.colors.len());
        for color in self.colors {
            let new_r = r_filter.consume(color.r);
            let new_g = g_filter.consume(color.g);
            let new_b = b_filter.consume(color.b);
            let new_w = w_filter.consume(color.w);
            new_colors.push_back(Color::new(new_r, new_g, new_b, new_w));
        }

        self.colors = new_colors;
        self
    }
}

impl Default for LedSequence {
    fn default() -> Self {
        Self {
            colors: VecDeque::new(),
            info: LedSequenceInfo {
                sequence_type: LedSequenceType::Color,
                name: "default".to_string(),
                duration: 0.0,
                repeat: false,
            },
            index: 0,
            repeat_start: 0,
        }
    }
}

impl Iterator for &mut LedSequence {
    type Item = Color;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.colors.len() {
            if self.info.repeat {
                self.index = self.repeat_start;
            } else {
                return None;
            }
        }
        let color = self.colors[self.index].clone();
        self.index += 1;
        Some(color)
    }
}
