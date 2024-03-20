mod server;
mod http;

use std::io;
use server::WebServer;

fn main() -> io::Result<()> {
    let mut webserver = WebServer::new();
    webserver.serve_http()
}
