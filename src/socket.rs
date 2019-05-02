use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::{thread, time};

#[derive(Debug)]
pub enum Error {
    Write,
    Read,
}

pub struct Socket {
    host: String,
    agent: String,
    connected: bool,
    stream: TcpStream,
    reader: BufReader<TcpStream>,
}

pub fn log(host: &str, agent: &str, message: &str) {
    println!(
        "{}",
        json::stringify(json::object! {
            "message" => message,
            "agent" => agent,
            "host" => host,
        })
    );
}

/// # Socket
///
/// The user interface for this library.
///
/// ```no_run
/// use wanbus::socket::Socket;
///
/// let host = "pubsub.pubnub.com:80";
/// let mut socket = Socket::new(host, "HTTP Agent");
/// ```
impl Socket {
    pub fn new(host: &str, agent: &str) -> Self {
        let stream = Socket::connect(host, agent);
        Self {
            host: host.into(),
            agent: agent.into(),
            connected: true,
            stream: stream.try_clone().expect("Unable to clone stream"),
            reader: BufReader::new(stream),
        }
    }

    pub fn log(&mut self, message: &str) {
        log(&self.host, &self.agent, message);
    }

    pub fn check_reconnect(&mut self) {
        if self.connected {
            return;
        }
        self.reconnect();
        self.connected = true;
    }

    /// ## Write Data
    ///
    /// Write string data to the stream.
    ///
    /// ```no_run
    /// use wanbus::socket::Socket;
    /// let host = "pubsub.pubnub.com:80";
    /// let mut socket = Socket::new(host, "HTTP Agent");
    /// let request = "GET / HTTP/1.1\r\nHost: pubnub.com\r\n\r\n";
    /// socket.write(request).expect("data written");
    /// ```
    pub fn write(&mut self, data: impl AsRef<str>) -> Result<usize, Error> {
        // Reconnect if not connected
        self.check_reconnect();

        let result = self.stream.write(data.as_ref().as_bytes());
        match result {
            Ok(size) => {
                if size > 0 {
                    return Ok(size);
                }
                self.log("No data has been written.");
                self.connected = false;
                Err(Error::Write)
            }
            Err(error) => {
                self.log(&format!("Unwrittable: {}", error));
                self.log(&format!("Disconnected: {}", error));
                self.connected = false;
                Err(Error::Write)
            }
        }
    }

    /// ## Read Line
    ///
    /// Read a line of data from the stream.
    ///
    /// ```no_run
    /// use wanbus::socket::Socket;
    /// let host = "pubsub.pubnub.com:80";
    /// let mut socket = Socket::new(host.into(), "HTTP Agent");
    /// let request = "GET / HTTP/1.1\r\nHost: pubnub.com\r\n\r\n";
    /// socket.write(request);
    /// let line = socket.readln();
    /// ```
    pub fn readln(&mut self) -> Result<String, Error> {
        // Reconnect if not connected
        self.check_reconnect();

        let mut line = String::new();
        let result = self.reader.read_line(&mut line);
        let size = result.unwrap_or_else(|_| 0);

        if size == 0 {
            self.connected = false;
            Err(Error::Read)?;
        }

        Ok(line)
    }

    /// ## Read Bytes
    ///
    /// Read specified amount of data from the stream.
    ///
    /// ```no_run
    /// use wanbus::socket::Socket;
    ///
    /// let host = "pubsub.pubnub.com:80";
    /// let mut socket = Socket::new(host.into(), "HTTP Agent");
    /// let request = "GET / HTTP/1.1\r\nHost: pubnub.com\r\n\r\n";
    /// socket.write(request).expect("data written");
    /// let data = socket.read(30).expect("data read"); // read 30 bytes
    /// println!("{}", data);
    /// ```
    pub fn read(&mut self, bytes: usize) -> Result<String, Error> {
        // Reconnect if not connected
        self.check_reconnect();

        let mut buffer = vec![0u8; bytes];
        let result = self.reader.read(&mut buffer);

        if result.is_err() {
            self.connected = false;
            Err(Error::Read)?;
        }

        Ok(String::from_utf8_lossy(&buffer).to_string())
    }

    /// ## Disconnect
    ///
    /// This will courteously turn off the connection of your socket.
    ///
    /// ```no_run
    /// use wanbus::socket::Socket;
    /// let host = "pubsub.pubnub.com:80";
    /// let mut socket = Socket::new(host.into(), "HTTP Agent");
    /// socket.disconnect();
    /// ```
    pub fn disconnect(&mut self) {
        self.stream.shutdown(Shutdown::Both).unwrap_or_default();
    }

    pub fn reconnect(&mut self) {
        thread::sleep(time::Duration::new(1, 0));
        let stream = Socket::connect(&self.host, &self.agent);
        self.connected = true;
        self.stream = stream.try_clone().expect("Unable to clone stream");
        self.reader = BufReader::new(stream);
    }

    fn connect(ip_port: &str, agent: &str) -> TcpStream {
        loop {
            // Open connection and send initialization data
            let host: String = ip_port.into();
            let error = match TcpStream::connect(host) {
                Ok(stream) => {
                    log(ip_port, agent, "Connected");
                    return stream;
                }
                Err(error) => error,
            };

            // Retry connection until the host becomes available
            log(ip_port, agent, &format!("{}", error));
            thread::sleep(time::Duration::new(1, 0));
        }
    }
}

#[cfg(test)]
mod socket_tests {
    use super::*;

    #[test]
    fn write_ok() {
        let host = "www.pubnub.com:80".into();
        let mut socket = Socket::new(host, "HTTP Agent");

        let request = "GET / HTTP/1.1\r\nHost: pubnub.com\r\n\r\n";
        let _ = socket.write(request).expect("data written");
    }

    #[test]
    fn read_ok() {
        let host = "www.pubnub.com:80".into();
        let mut socket = Socket::new(host, "HTTP Agent");

        let request = "GET / HTTP/1.1\r\nHost: pubnub.com\r\n\r\n";
        socket.write(request).expect("data written");

        let result = socket.readln();
        assert!(result.is_ok());

        let data = result.expect("data");
        assert!(data.len() > 0);

        let result = socket.readln();
        assert!(result.is_ok());

        let data = result.expect("data");
        assert!(data.len() > 0);
    }
}
