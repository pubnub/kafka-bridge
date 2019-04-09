// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Dependencies
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
use std::net::{TcpStream, TcpListener};
use std::io::{Write};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let response = b"Hello World";
                stream.write(response).expect("Response failed");
            }
            Err(e) => {
                println!("Unable to connect: {}", e);
            }
        }
    }
}
