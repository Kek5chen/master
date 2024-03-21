use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{self, Error, ErrorKind, Read, Write};
use std::io::ErrorKind::WouldBlock;

use crate::http::HTTPMessage;

const MAX_READ_RETRIES: u32 = 1000;

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
        client.set_nonblocking(true)?;

        match self.read_request(client) {
            Ok(msg) => self.respond(client, &msg)?,
            Err(e) => self.respond_error(client, &e)?
        };

        client.flush()?;
        client.shutdown(Shutdown::Both)
    }

    fn read_request(&self, client: &mut TcpStream) -> io::Result<HTTPMessage> {
        let mut buffer = [0; 1024];
        let mut content = Vec::new();
        let mut read_retries = 0;

        loop {
            match client.read(&mut buffer) {
                Ok(0) => break,
                Ok(read_num) => content.extend_from_slice(&buffer[..read_num]),
                Err(e) if e.kind() == WouldBlock => {
                    read_retries += 1;
                    if read_retries >= MAX_READ_RETRIES {
                        println!("Maximum read tries were reached.. cancelling..");
                        break;
                    }
                    if !content.is_empty() {
                        break;
                    }
                },
                Err(e) => return Err(e),
            };
        }

        if content.is_empty() {
            return Err(Error::new(ErrorKind::InvalidData, "The request was empty"));
        }

        let message = String::from_utf8(content)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid UTF-8 sequence"))?;
        let message = HTTPMessage::parse_request(&message);

        match message {
            Ok(message) => Ok(message),
            Err(e) => Err(Error::new(ErrorKind::InvalidData, "The request was invalid."))
        }
    }

    fn respond(&self, client: &mut TcpStream, request: &HTTPMessage) -> io::Result<()> {
        println!("[{} on {}] {} for path {}",
                 &client.peer_addr()?,
                 request.get("User-Agent").unwrap_or(&String::from("No User Agent")),
                 &request.request_type,
                 &request.path);
        let response = HTTPMessage::new().make_response();
        client.write_all(response.as_bytes())
    }

    fn respond_error(&self, client: &mut TcpStream, error: &Error) -> io::Result<()> {
        let mut response = "HTTP/1.1 500 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<h1>oh nonono... Erororer!!\n\n".to_string();

        response += error.to_string().as_str();
        response += "</h1>";
        client.write_all(response.as_bytes())
    }
}

