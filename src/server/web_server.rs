use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{self, Error, ErrorKind, Read, Write};
use std::time::Duration;

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
        client.set_read_timeout(Some(Duration::from_secs(10)));
        println!("Accepted client {}", client.peer_addr()?);

        match self.read_request(client) {
            Ok(msg) => self.respond(client, &msg)?,
            Err(e) => self.respond_error(client, &e)?
        };


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
        let mut buffer = [0; 1024];
        let mut content = Vec::new();
        // TODO: Will crash if buffer size was overrun
        let read_num = client.read(&mut buffer)?;

        if !buffer[..read_num].ends_with(b"\r\n\r\n") {
            return Err(Error::new(ErrorKind::InvalidData, "The request header was malformed."));
        }

        let headers = String::from_utf8(content).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 sequence"))?;
        let message = HTTPMessage::from(&headers);
        println!("Client requested path {}", &message.path);
        Ok(message)
    }

    fn respond(&self, client: &mut TcpStream, request: &HTTPMessage) -> io::Result<()> {
        let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<h1>literally mowserver</h1>";
        client.write_all(response.as_bytes())
    }

    fn respond_error(&self, client: &mut TcpStream, error: &Error) -> io::Result<()> {
        let mut response = "HTTP/1.1 500 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<h1>oh nonono... Erororer!!\n\n".to_string();

        response += error.to_string().as_str();
        response += "</h1>";
        client.write_all(response.as_bytes())
    }
}

