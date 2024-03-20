use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{self, Read, Write, ErrorKind};

use crate::server::HTTPMessage;

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
        let server = self.server
            .as_ref()
            .expect("accept_clients was called without server being set previously");
        loop {
            for client in server.incoming() {
                match client {
                    Ok(mut client) => self.handle_client(&mut client).expect("Couldn't handle client"),
                    Err(e) => eprintln!("Connection failed: {e}"),
                };
            }
        }
    }

    fn handle_client(&self, client: &mut TcpStream) -> io::Result<()> {
        println!("Accepted client {}", client.peer_addr()?);
        let request = self.read_request(client)?;
        println!("Client requested path {}", request.path);
        self.respond(client, &request)?;
        client.flush()?;
        client.shutdown(Shutdown::Both)
    }

    fn read_request(&self, client: &mut TcpStream) -> io::Result<HTTPMessage> {
        let mut message = self.read_request_header(client)?;
        let content_length = message
            .get("Content-Length")
            .and_then(|c| c.parse::<usize>().ok())
            .unwrap_or(0);
        if content_length > 0 {
            let mut body = vec![0u8; content_length];
            client.read_exact(&mut body);
            message.data = String::from_utf8(body).unwrap_or_else(|_| String::new());
        }
        Ok(message)
    }

    fn read_request_header(&self, client: &mut TcpStream) -> io::Result<HTTPMessage> {
        let mut buffer = [0; 1024]; // Example size, adjust as necessary
        let mut content = Vec::new();
        let mut read_len = client.read(&mut buffer)?;

        // Basic loop to read headers; consider improving for robustness
        while read_len > 0 && !content.ends_with(b"\r\n\r\n") {
            content.extend_from_slice(&buffer[..read_len]);
            read_len = match client.read(&mut buffer) {
                Ok(size) => size,
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            };
        }

        let headers = String::from_utf8(content).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 sequence"))?;
        Ok(HTTPMessage::from(&headers))
    }

    fn respond(&self, client: &mut TcpStream, request: &HTTPMessage) -> io::Result<()> {
        let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<h1>literally mowserver</h1>";
        client.write_all(response.as_bytes())
    }
}

