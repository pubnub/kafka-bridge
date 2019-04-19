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
pub struct Socket {
    host: String,
    stream: TcpStream,
    reader: BufReader<TcpStream>,
}

pub struct Line {
    ok: bool,
    size: usize,
    data: String,
}

impl Socket {
    pub fn new(host: &str) -> Self {
        let stream = Self::connect(host);

        Self {
            host: host.into(),
            stream: stream.try_clone().unwrap(),
            reader: BufReader::new(stream.try_clone().unwrap()),
        }
    }

    fn log(data: &str) {
        eprintln!("{}", data);
    }

    fn connect(host: &str) -> TcpStream {
        loop {
            let connection = TcpStream::connect(&host);
            let error = match connection {
                Ok(stream) => return stream,
                Err(error) => error,
            };

            // unable to connect, retry
            Self::log(&json::stringify(object! {
                "message" => "Host unreachable.",
                "host" => host,
                "error" => format!("{}", error),
            }));

            thread::sleep(time::Duration::new(1, 0));
        }
    }

    pub fn disconnect(&mut self) -> Result<(), std::io::Error>{
        self.stream.shutdown(Shutdown::Both)
    }

    fn reconnect(&mut self) {
        Self::log(&json::stringify(object! {
            "message" => "Lost connection, reconnecting.",
            "host" => self.host.clone(),
        }));
        thread::sleep(time::Duration::new(1, 0));
        self.stream = Self::connect(&self.host);
        self.reader = BufReader::new(self.stream.try_clone().unwrap());
    }

    pub fn write(&mut self, data: &str) -> Result<usize, usize> {
        let result = self.stream.write(data.as_bytes());
        match result {
            Ok(size) => Ok(size),
            Err(error) => {
                Self::log(&json::stringify(object! {
                    "message" => "Lost connection, reconnecting.",
                    "host" => self.host.clone(),
                    "error" => format!("{}", error),
                }));
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
        let socket = Socket::new(host);
        assert!(socket.host == host);
    }

    #[test]
    fn socket_write_ok() {
        let host = "www.pubnub.com:80";
        let mut socket = Socket::new(host);

        let request = "GET / HTTP/1.1\r\nHost: pubnub.com\r\n\r\n";
        let result = socket.write(&request);
        assert!(result.is_ok());

        let size = result.unwrap();
        assert!(size > 0);
    }

    #[test]
    fn socket_read_and_write_ok() {
        let host = "www.pubnub.com:80";
        let mut socket = Socket::new(host);
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
