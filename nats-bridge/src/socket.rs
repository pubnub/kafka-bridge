// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Dependencies
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
//use tokio::io;
use tokio::net::TcpStream;
use tokio::prelude::*;

use failure::Fail;

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// All the ways in which this lib will fail
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
#[derive(Debug, Fail)]
pub enum SocketError {
    #[fail(display = "Missing HOST configuration.")]
    MissingHost,

    #[fail(display = "Unparseable.")]
    ParseError(#[cause] std::net::AddrParseError),

    #[fail(display = "IO Error.")]
    IOError(#[cause] std::io::Error),

    #[fail(display = "Invalid HOST `{}` configuration.", _0)]
    Unconnectable(String),

    #[fail(display = "Missing PORT configuration.")]
    MissingPort,

    #[fail(display = "Something done broked.")]
    General,
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Structures
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/*
#[deriving(Decodable, Encodable)]
struct NATSConnectionSettings {
    user: String,
    password: String,
    token: String
}
*/


// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// 
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
impl From<std::net::AddrParseError> for SocketError {
    fn from(error: std::net::AddrParseError) -> Self {
        SocketError::ParseError(error)
    }
}
impl From<std::io::Error> for SocketError {
    fn from(error: std::io::Error) -> Self {
        SocketError::IOError(error)
    }
}


// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Persistent Socket Lib
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
pub fn connect(host:&'static str) -> Result<impl Future, SocketError> {
    let hoststring = host.to_owned();
    let addr = host.parse().map_err(SocketError::ParseError)?;

    let client = TcpStream::connect(&addr)
    .and_then( |_stream| {
        println!("Connected");
        Ok(())
    })
    .map_err(SocketError::IOError);

    Ok(client)
}

/*
pub fn send() -> Result<(), SocketError> {
    println!("Data Sent");
    //bail!("Unable to send DATA to HOST.");
    Ok(())
}

pub fn receive() -> Result<(), SocketError> {
    println!("Data Received");
    //bail!("Unable to receive DATA from HOST.");
    Ok(())
}

pub fn disconnect() -> Result<(), SocketError> {
    println!("Disconnected");
    //bail!("Unable to disconnect from HOST.");
    Ok(())
}
*/

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Tests
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
#[cfg(test)]
mod tests {
    use crate::socket;
    use super::*;

    /*
    #[test]
    fn test_connect_err() {
        let host = "255.255.255.255:5555";
        assert!(
            socket::connect(host).is_err(),
            "Invalid host is unreachable."
        );
    }
    */

    #[test]
    fn test_connect_ok() {
        //let host = "psdsn.pubnub.com:80";
        let host = "1.1.1.1:80";
        assert!(
            socket::connect(host).or_else(|e| {

                eprintln!("{} {}", "error:", e);
                for cause in Fail::iter_causes(&e) {
                    eprintln!("{} {}", "caused by:", cause);
                }

                Err(e)
            }).is_ok(),
            "Connected to host."
        );
        //assert_eq!(socket::connect(), ());
        //assert_eq!(socket::send(), ());
        //assert_eq!(socket::receive(), ());
        //assert_ne!(socket::disconnect(), ());
        //assert!(socket::disconnect().is_err(), "db file should not exist");
        assert_eq!(2 + 2, 4);
    }
}
