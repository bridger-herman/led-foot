#[macro_use]
extern crate stdweb;

pub mod led_ui;

fn main() {
    let mut ui = led_ui::LedUi::default();
    ui.setup();
}
