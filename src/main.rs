#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate stdweb;

#[macro_use]
pub mod led_system;

pub mod color;
pub mod led_ui;

fn main() {
    let mut ui = led_ui::LedUi::default();
    ui.setup();
}
