// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Dependencies
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
use std::io::{Read, Write};
use std::net::TcpStream;

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Main
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
fn main() -> std::io::Result<()> {
    let mut socket = TcpStream::connect("psdsn.pubnub.com:80")?;
    let mut buffer = String::new();

    let result = socket.write(b"GET /time/0 HTTP/1.1\r\nHost: pubnub.com\r\n\r\n");

    socket.read_to_string(&mut buffer)?;
    println!("{}",buffer);
    Ok(())
}

