// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Imports
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
use std::io::{BufRead, BufReader, Write};
use std::net::{Shutdown, TcpStream};
use std::{thread, time};

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/// # Socket Policy
///
/// Describes what actions a Socket will take for various situations.
///
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
pub trait SocketPolicy {
    // Attributes
    fn host(&self) -> &str;

    // Events
    fn initialized(&self) {}
    fn connected(&self) {}
    fn disconnected(&self, error: &str) {}
    fn unreachable(&self, error: &str) {}

    // Behaviors
    fn data_on_connect(&self) -> String;
    fn retry_delay_after_disconnected(&self) -> u64;
    fn retry_delay_when_unreachable(&self) -> u64;
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/// # Socket
///
/// The user interface to this library will be accessed via the Socet struct.
///
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
pub struct Socket {
    pub client: String,
    pub host: String,
    policy: Box<SocketPolicy>,
    stream: TcpStream,
    reader: BufReader<TcpStream>,
}

impl Socket {
    pub fn new<P: SocketPolicy + 'static>(
        client: &str,
        host: &str,
        policy: P,
    ) -> Self {
        policy.initialized();
        let stream = Self::connect(&policy);
        policy.connected();

        Self {
            client: client.into(),
            host: host.into(),
            policy: Box::new(policy),
            stream: stream.try_clone().unwrap(),
            reader: BufReader::new(stream.try_clone().unwrap()),
        }
    }

    pub fn disconnect(&mut self) {
        self.stream.shutdown(Shutdown::Both).expect("Shutdown");
    }

    fn connect<P: SocketPolicy + 'static>(policy: &P) -> TcpStream {
        let retry_delay = policy.retry_delay_when_unreachable();
        loop {
            let host: String = policy.host().into();
            let connection = TcpStream::connect(host);
            let error = match connection {
                Ok(stream) => return stream,
                Err(error) => error,
            };

            // Retry connection until the host becomes available
            policy.unreachable(&format!("{}", error));
            thread::sleep(time::Duration::new(retry_delay, 0));
        }
    }

    pub fn write(&mut self, data: &str) -> Result<usize, usize> {
        let result = self.stream.write(data.as_bytes());
        match result {
            Ok(size) => Ok(size),
            Err(error) => {
                self.policy.disconnected(&format!("{}",error));
                // TODO 
                // TODO reconnect
                // TODO resend
                // TODO 
                Err(0)
            }
        }
    }
}

/*
impl Socket {
    pub fn new(client: &str, host: &str) -> Self {
        let client = Client::new(client, host);
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


    pub fn readln(&mut self) -> Line {
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
        let client = "http client";
        let socket = Socket::new(client, host);
        assert!(socket.client.host == host);
        assert!(socket.client.client == client);
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
*/
