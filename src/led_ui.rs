//! Controls for the actual UI on the LEDs

use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web::{
    document, event::ChangeEvent, html_element::InputElement,
    INonElementParentNode,
};

use crate::color::Color;

const DEFAULT_RANGES: [&str; 5] = ["red", "green", "blue", "white", "bright"];

pub struct LedUi {
    input_range_names: Vec<String>,
}

impl LedUi {
    pub fn setup(&mut self) {
        for range in &self.input_range_names {
            let input: InputElement = document()
                .get_element_by_id(&format!("input-range-{}", range))
                .unwrap()
                .try_into()
                .unwrap();
            input.set_raw_value(&format!("{}", u8::default()));
            input.add_event_listener(move |_event: ChangeEvent| {
                let mut color = Color::default();
                let range: InputElement = document()
                    .get_element_by_id("input-range-red")
                    .unwrap()
                    .try_into()
                    .unwrap();
                js! {
                    console.log(@{range.raw_value()});
                }
                color.r = range.raw_value().parse::<u8>().unwrap();
                let range: InputElement = document()
                    .get_element_by_id("input-range-green")
                    .unwrap()
                    .try_into()
                    .unwrap();
                color.g = range.raw_value().parse::<u8>().unwrap();
                let range: InputElement = document()
                    .get_element_by_id("input-range-blue")
                    .unwrap()
                    .try_into()
                    .unwrap();
                color.b = range.raw_value().parse::<u8>().unwrap();
                let range: InputElement = document()
                    .get_element_by_id("input-range-white")
                    .unwrap()
                    .try_into()
                    .unwrap();
                color.w = range.raw_value().parse::<u8>().unwrap();

                js! {
                    console.log("ranges: " + @{color.rgb_to_css()});
                }

                led_system!().set_color(&color);
                led_system!().update();
            });
        }
    }
}

impl Default for LedUi {
    fn default() -> Self {
        Self {
            input_range_names: DEFAULT_RANGES
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}
