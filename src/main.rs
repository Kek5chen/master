mod app;

use std::process::exit;
use crate::app::App;

fn main() {
    let mut app = match App::new("Funky App", 800, 600) {
        Err(e) => { eprintln!("{}", e); exit(1); },
        Ok(app) => app,
    };
    app.run()
}
