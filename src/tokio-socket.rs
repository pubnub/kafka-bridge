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

    #[fail(display = "Unparseable address.")]
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
// Error Type Conversions
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
// Connection Lib
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
pub struct Connection {
    host: String,
    socket: String,
}

impl Connection {
    pub fn new(host: String) -> Connection {
        Connected {
            host
        }
    }

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

    fn resolve_addr(&self, host: &str) -> Result<SocketAddr, String> {
        let mut addr_iter = match host.to_socket_addrs() {
            Ok(addr_iter) => addr_iter,
            Err(e) => return Err(format!("Invalid host {:?}: {:?}", host, e)),
        };
        match addr_iter.next() {
            None => Err(format!("No addresses found for host: {:?}", host)),
            Some(addr) => Ok(addr),
        }
    }

    fn send(&self, host: String, path: String) -> impl Future<Item=(), Error=()> {
	let mut addr_iter = host.to_socket_addrs().unwrap();
	let addr = match addr_iter.next() {
	    None => panic!("DNS resolution failed"),
	    Some(addr) => addr,
	};
	let req_body = format!(
	    "GET {} HTTP/1.1\r\nHost: {}:80\r\nConnection: close\r\n\r\n",
	    path,
	    host,
	    );

	TcpStream::connect(&addr)
        .and_then(|stream| {
            write_all(stream, req_body).and_then(|(stream, _body)| {
                let buffer = vec![];
                read_to_end(stream, buffer).and_then(|(_stream, buffer)| {
                    File::create(filename).and_then(|mut file| {
                        file.write_all(&buffer)
                    })
                })
            })
        }).map_err(|e| eprintln!("Error occured: {:?}", e))
    }
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
