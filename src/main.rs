use std::{net::{TcpListener, TcpStream, SocketAddrV4}, io::{Write, Read}};
use std::io;

#[allow(unused)]
pub struct WebServer {
    started_http: bool,
    server: Option<TcpListener>,
}

#[allow(unused)]
impl Default for WebServer {
    fn default() -> Self {
        WebServer::new()
    }
}

#[allow(unused)]
impl WebServer {
    pub fn new() -> Self {
        WebServer {
            started_http: false,
            server: None,
        }
    }

    pub fn serve_http(&mut self) -> io::Result<()> {
        self.started_http = true;
        self.server = Some(TcpListener::bind("127.0.0.1:8080")?);

        self.accept_clients();

        Ok(())
    }

    fn accept_clients(&self) -> io::Result<()> {
        let server = self.server.as_ref().expect("accept_clients was called without server being set previously");
        loop {
            for client in server.incoming() {
                match client {
                    Ok(mut client) => {
                        self.handle_client(client);
                    },
                    Err(e) => eprintln!("Connection failed: {e}"),
                }
            }
        }
    }

    fn handle_client(&self, mut client: TcpStream) -> io::Result<()> {
        println!("Accepted client {}", client.peer_addr()?);
        let mut buffer = Vec::new();
        client.read(&mut buffer);

        let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<h1>literally mowserver</h1>";
        client.write_all(response.as_bytes())?;
        client.flush()
    }
}

fn main() -> io::Result<()> {
    let mut webserver = WebServer::new();
    webserver.serve_http()
}
