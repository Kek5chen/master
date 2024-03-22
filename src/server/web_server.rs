use std::fs::File;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{self, Error, ErrorKind, Read, Write};
use std::io::ErrorKind::WouldBlock;
use std::time::{Duration, SystemTime};

use crate::http::HTTPMessage;
use crate::html::{RESPONSE_403, RESPONSE_404, RESPONSE_INVALID, RESPONSE_SHUTDOWN};

const VERSION: &str = "0.1.0";

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
    const READ_TIMEOUT: Duration = Duration::from_secs(10);

    pub fn new() -> Self {
        WebServer {
            started_http: false,
            server: None,
        }
    }

    pub fn serve_http(&mut self, port: u16) -> io::Result<()> {
        self.started_http = true;
        self.server = Some(TcpListener::bind(format!("0.0.0.0:{port}"))?);
        println!("Started webserver on 0.0.0.0:{port}");

        self.accept_clients();

        println!("Stopping webserver...");
        Ok(())
    }

    fn accept_clients(&self) -> io::Result<()> {
        let server = self.server
            .as_ref()
            .expect("accept_clients was called without server being set previously");
        for client in server.incoming() {
            match client {
                Ok(mut client) => {
                    match self.handle_client(&mut client) {
                        Ok(_) => (),
                        Err(e) if e.kind() == ErrorKind::Other => {
                            println!("{e}");
                            return Err(e);
                        },
                        Err(e) => {
                            eprintln!("Error handling client: {e}");
                            continue;
                        }
                    }
                },
                Err(e) => eprintln!("Connection failed: {e}"),
            }
        }
        Ok(())
    }

    fn handle_client(&self, client: &mut TcpStream) -> io::Result<()> {
        client.set_nonblocking(true)?;

        match self.read_request(client) {
            Ok(msg) => {
                if msg.path == "end_server_mow" {
                    let error: Error = Error::new(ErrorKind::Other, "Server shutdown requested");
                    self.respond_error(client, &msg, &error)?;
                    return Err(error);
                }
                self.respond(client, &msg)?
            },
            Err(e) => self.respond_invalid(client, &e)?
        };

        client.flush()?;
        client.shutdown(Shutdown::Both)
    }

    fn read_request(&self, client: &mut TcpStream) -> io::Result<HTTPMessage> {
        let mut buffer = [0; 1024];
        let mut content = Vec::new();
        let mut started_reading = SystemTime::now();

        loop {
            match client.read(&mut buffer) {
                Ok(0) => break,
                Ok(read_num) => content.extend_from_slice(&buffer[..read_num]),
                Err(e) if e.kind() == WouldBlock => {
                    if started_reading.elapsed().unwrap() > Self::READ_TIMEOUT {
                        println!("Read timed out after {} seconds.. cancelling..", Self::READ_TIMEOUT.as_secs());
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

        let mut file = match File::open(&request.path) {
            Ok(file) => file,
            Err(e) => return self.respond_error(client, request, &e),
        };

        let mut response = HTTPMessage::new();
        response.path = request.path.clone();

        file.read_to_end(&mut response.bin_data);

        client.write_all(&response.make_response())
    }

    fn respond_invalid(&self, client: &mut TcpStream, error: &Error) -> io::Result<()> {
        println!("[{}] Invalid request", &client.peer_addr()?);
        let mut response = HTTPMessage::new();

        response.body = Self::replace_placeholders(RESPONSE_INVALID, None);
        response.status_code = 400;

        client.write_all(&response.make_response())
    }

    fn respond_error(&self, client: &mut TcpStream, request: &HTTPMessage, error: &Error)
        -> io::Result<()> {
        let mut response = HTTPMessage::new();

        response.body = match error.kind() {
            ErrorKind::PermissionDenied => Self::replace_placeholders(RESPONSE_403, Some(request)),
            ErrorKind::Other => Self::replace_placeholders(RESPONSE_SHUTDOWN, Some(request)),
            _ => Self::replace_placeholders(RESPONSE_404, Some(request)),
        };
        response.status_code = match error.kind() {
            ErrorKind::NotFound => 404,
            ErrorKind::PermissionDenied => 403,
            ErrorKind::Other => 200,
            _ => 500,
        };
        response.header.insert("Content-Type".to_string(), "text/html".to_string());

        client.write_all(&response.make_response())
    }

    fn replace_placeholders(text: &str, request: Option<&HTTPMessage>) -> String {
        let mut new_text = text.replace("#%VERSION%#", VERSION);
        let request = match request {
            Some(request) => request,
            None => return new_text,
        };

        new_text.replace("#%PATH%#", &request.path)
    }
}
