use std::sync::Mutex;

use stdweb::unstable::TryInto;
use stdweb::web::{document, HtmlElement, INonElementParentNode};

use crate::color::Color;

#[macro_export]
macro_rules! led_system {
    () => {
        crate::led_system::LED_SYSTEM.try_lock().unwrap()
    };
}

lazy_static! {
    pub static ref LED_SYSTEM: Mutex<LedSystem> =
        Mutex::new(LedSystem::default());
}

#[derive(Default)]
pub struct LedSystem {
    current_color: Color,
}

impl LedSystem {
    pub fn set_color(&mut self, color: &Color) {
        self.current_color = color.clone();
    }

    pub fn update(&mut self) {
        let color_preview: HtmlElement = document()
            .get_element_by_id("color-preview")
            .unwrap()
            .try_into()
            .unwrap();
        let white_preview: HtmlElement = document()
            .get_element_by_id("white-preview")
            .unwrap()
            .try_into()
            .unwrap();
        js! {
            @{color_preview}.style.backgroundColor = @{self.current_color.rgb_to_css()};
            @{white_preview}.style.backgroundColor = @{self.current_color.white_to_css()};
        }
    }
}
