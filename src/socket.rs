// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Imports
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
use json::object;
use std::io::{BufRead, BufReader, Write};
use std::net::{Shutdown, TcpStream};
use std::{thread, time};

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Socket Class and Struct
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
pub(crate) struct Client {
    pub(crate) name: String,
    pub(crate) host: String,
}

pub(crate) struct Socket {
    pub(crate) client: Client,
    stream: TcpStream,
    reader: BufReader<TcpStream>,
}

pub(crate) struct Line {
    pub(crate) ok: bool,
    pub(crate) size: usize,
    pub(crate) data: String,
}

impl Client {
    fn new(name: &str, host: &str) -> Self {
        Self {
            name: name.into(),
            host: host.into(),
        }
    }

    fn log(&self, message: &str) {
        eprintln!("{}", json::stringify(object!{ 
            "message" => message,
            "host" => self.host.clone(),
            "name" => self.name.clone(),
        }));
    }

    fn connect(&self) -> TcpStream {
        loop {
            let connection = TcpStream::connect(&self.host);
            let error = match connection {
                Ok(stream) => return stream,
                Err(error) => error,
            };

            self.log(&format!("Host unreachable: {}", error));
            thread::sleep(time::Duration::new(1, 0));
        }
    }
}

impl Socket {
    pub fn new(name: &str, host: &str) -> Self {
        let client = Client::new(name, host);
        let stream = client.connect();

        Self {
            client,
            stream: stream.try_clone().expect("Failed to clone TCPStream"),
            reader: BufReader::new(stream),
        }
    }

    fn log(&self, message: &str) {
        self.client.log(message)
    }

    pub fn disconnect(&mut self) -> Result<(), std::io::Error>{
        self.stream.shutdown(Shutdown::Both)
    }

    fn reconnect(&mut self) {
        self.log("Lost connection, reconnecting.");
        thread::sleep(time::Duration::new(1, 0));
        self.stream = self.client.connect();
        self.reader = BufReader::new(self.stream.try_clone().unwrap());
    }

    pub fn write(&mut self, data: &str) -> Result<usize, usize> {
        let result = self.stream.write(data.as_bytes());
        match result {
            Ok(size) => Ok(size),
            Err(error) => {
                self.log(&format!(
                    "Lost connection, reconnecting shortly: {}",
                    error
                ));
                Err(0)
            }
        }
    }

    pub fn read_line(&mut self) -> Line {
        loop {
            let mut line = String::new();
            let result   = self.reader.read_line(&mut line);
            let size     = result.unwrap_or_else( |_| 0 );

            if size == 0 {
                Self::reconnect(self);
                break Line { ok: false, size: size, data: line };
            }

            break Line { ok: true, size: size, data: line };
        }
    }
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Tests
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
#[cfg(test)]
mod socket_tests {
    use super::*;

    #[test]
    fn socket_connect_ok() {
        let host = "www.pubnub.com:80";
        let name = "http client";
        let socket = Socket::new(name, host);
        assert!(socket.client.host == host);
        assert!(socket.client.name == name);
    }

    #[test]
    fn socket_write_ok() {
        let host = "www.pubnub.com:80";
        let mut socket = Socket::new("http client", host);

        let request = "GET / HTTP/1.1\r\nHost: pubnub.com\r\n\r\n";
        let result = socket.write(&request);
        assert!(result.is_ok());

        let size = result.unwrap();
        assert!(size > 0);
    }

    #[test]
    fn socket_read_and_write_ok() {
        let host = "www.pubnub.com:80";
        let mut socket = Socket::new("http client", host);
        let request = "GET / HTTP/1.1\r\nHost: pubnub.com\r\n\r\n";
        let result = socket.write(&request);
        assert!(result.is_ok());

        let line = socket.read_line();
        assert!(line.ok);
        assert!(line.size > 0);

        let line = socket.read_line();
        assert!(line.ok);
        assert!(line.size > 0);
    }
}
