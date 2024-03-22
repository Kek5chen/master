mod server;
mod http;
mod html;

use std::io;
use server::WebServer;

fn main() -> io::Result<()> {
    let mut webserver = WebServer::new();
    match webserver.serve_http(80) {
        Err(_) => {
            match webserver.serve_http(8080) {
                Err(e) => {println!("Failed to run webserver on port 80 or 8080"); Err(e) },
                _ => Ok(()),
            }
        },
        _ => { Ok(()) },
    }
}
