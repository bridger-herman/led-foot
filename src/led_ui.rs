//! Controls for the actual UI on the LEDs

use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web::{
    document, event::ClickEvent, HtmlElement, html_element::InputElement,
    INonElementParentNode,
};

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
            let range_clone = range.clone();
            input.add_event_listener(move |event: ClickEvent| {
                let target: InputElement =
                    event.target().unwrap().try_into().unwrap();
                js! {
                    console.log(@{&range_clone} + @{target.raw_value()});
                }
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
