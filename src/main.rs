#[macro_use]
extern crate stdweb;

use stdweb::traits::*;

use stdweb::web::{
    document, event::ClickEvent, html_element::InputElement,
    INonElementParentNode,
};

use stdweb::unstable::TryInto;

fn main() {
    for range in &["red", "green", "blue", "white", "bright"] {
        let input: InputElement = document()
            .get_element_by_id(&format!("input-range-{}", range))
            .unwrap()
            .try_into()
            .unwrap();
        input.add_event_listener(move |event: ClickEvent| {
            let target: InputElement =
                event.target().unwrap().try_into().unwrap();
            js! {
                console.log(@{range} + @{target.raw_value()});
            }
        });
    }
}
