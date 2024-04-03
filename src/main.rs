mod app;

use std::ffi::CStr;
use std::thread::sleep_ms;
use crate::app::App;

fn main() {
    App::new("Funky App", 800, 600).expect("oh no");
    sleep_ms(10000)
}
