mod app;

use std::time::Duration;
use crate::app::App;

fn main() {
    let app = App::new("Funky App", 800, 600)
        .expect("oh no..?");
    std::thread::sleep(Duration::from_secs(10));
    drop(app);
    std::thread::sleep(Duration::from_secs(10));
}
