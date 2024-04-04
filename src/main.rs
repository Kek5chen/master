mod app;

use std::process::exit;
use std::time::Duration;
use crate::app::App;

fn main() {
    let app = match App::new("Funky App", 800, 600) {
        Err(e) => { eprintln!("{}", e); exit(1); },
        _ => (),
    };
    std::thread::sleep(Duration::from_secs(10));
    drop(app);
    std::thread::sleep(Duration::from_secs(10));
}
