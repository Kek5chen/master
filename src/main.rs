use std::{net::{TcpListener, TcpStream, SocketAddrV4}, io::{Write, Read}};
use std::io;

fn main() -> io::Result<()> {
    let mut server: TcpListener = TcpListener::bind("127.0.0.1:8080").unwrap();

    loop {
        for client in server.incoming() {
            match client {
                Ok(mut client) => {
                    let mut buffer = [0; 1024];
                    client.read(&mut buffer);

                    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<h1>literally mowserver</h1>";
                    client.write_all(response.as_bytes())?;
                    client.flush()?;
                },
                Err(e) => eprintln!("Connection failed: {e}"),
            }
        }
    }
}
