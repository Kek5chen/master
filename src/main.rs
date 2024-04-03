mod app;

use std::time::Duration;
use crate::app::App;

fn main() {
    App::new("Funky App", 800, 600).expect("oh no");
    std::thread::sleep(Duration::from_secs(10));
}
