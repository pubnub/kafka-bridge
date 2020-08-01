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
    timeout: u64,
    stream: TcpStream,
    reader: BufReader<TcpStream>,
}

pub fn log(host: &str, agent: &str, info: &str) {
    println!(
        "{}",
        json::stringify(json::object! {
            "info" => info,
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
/// use kafka_bridge::socket::Socket;
///
/// let host = "pubsub.pubnub.com:80";
/// let mut socket = Socket::new(host, "HTTP Agent", 5);
/// ```
impl Socket {
    #[must_use]
    pub fn new(host: &str, agent: &str, timeout: u64) -> Self {
        let stream = Socket::connect(host, agent, timeout);
        Self {
            host: host.into(),
            agent: agent.into(),
            timeout,
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
    /// use kafka_bridge::socket::Socket;
    /// let host = "pubsub.pubnub.com:80";
    /// let mut socket = Socket::new(host, "HTTP Agent", 5);
    /// let request = "GET / HTTP/1.1\r\nHost: pubnub.com\r\n\r\n";
    /// socket.write(request).expect("data written");
    /// ```
    ///
    /// # Errors
    ///
    /// This function can return [`Error::Write`] on unsuccessful write.
    pub fn write(&mut self, data: impl AsRef<str>) -> Result<usize, Error> {
        // Reconnect if not connected
        self.check_reconnect();

        // Log Write Output
        self.log(data.as_ref());

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
    /// use kafka_bridge::socket::Socket;
    /// let host = "pubsub.pubnub.com:80";
    /// let mut socket = Socket::new(host.into(), "HTTP Agent", 5);
    /// let request = "GET / HTTP/1.1\r\nHost: pubnub.com\r\n\r\n";
    /// socket.write(request);
    /// let line = socket.readln();
    /// ```
    ///
    /// # Errors
    ///
    /// This function can return [`Error::Read`] on unsuccessful read.
    pub fn readln(&mut self) -> Result<String, Error> {
        // Reconnect if not connected
        self.check_reconnect();

        let mut line = String::new();
        let size = match self.reader.read_line(&mut line) {
            Ok(size) => size,
            Err(_error) => {
                self.connected = false;
                return Err(Error::Read);
            }
        };

        if size == 0 {
            self.connected = false;
            return Err(Error::Read);
        }

        Ok(line)
    }

    /// ## Read Bytes
    ///
    /// Read specified amount of data from the stream.
    ///
    /// ```no_run
    /// use kafka_bridge::socket::Socket;
    ///
    /// let host = "pubsub.pubnub.com:80";
    /// let mut socket = Socket::new(host.into(), "HTTP Agent", 5);
    /// let request = "GET / HTTP/1.1\r\nHost: pubnub.com\r\n\r\n";
    /// socket.write(request).expect("data written");
    /// let data = socket.read(30).expect("data read"); // read 30 bytes
    /// println!("{}", data);
    /// ```
    ///
    /// # Errors
    ///
    /// This function can return [`Error::Read`] on unsuccessful read.
    pub fn read(&mut self, bytes: usize) -> Result<String, Error> {
        // Reconnect if not connected
        self.check_reconnect();

        let mut buffer = vec![0_u8; bytes];
        let size = match self.reader.read(&mut buffer) {
            Ok(size) => size,
            Err(_error) => {
                self.connected = false;
                return Err(Error::Read);
            }
        };

        if size == 0 {
            self.connected = false;
            return Err(Error::Read);
        }

        Ok(String::from_utf8_lossy(&buffer).to_string())
    }

    /// ## Disconnect
    ///
    /// This will courteously turn off the connection of your socket.
    ///
    /// ```no_run
    /// use kafka_bridge::socket::Socket;
    /// let host = "pubsub.pubnub.com:80";
    /// let mut socket = Socket::new(host.into(), "HTTP Agent", 5);
    /// socket.disconnect();
    /// ```
    pub fn disconnect(&mut self) {
        self.stream.shutdown(Shutdown::Both).unwrap_or_default();
    }

    pub fn reconnect(&mut self) {
        thread::sleep(time::Duration::new(1, 0));
        self.log("Reconnecting");
        let stream = Socket::connect(&self.host, &self.agent, self.timeout);
        self.connected = true;
        self.stream = stream.try_clone().expect("Unable to clone stream");
        self.reader = BufReader::new(stream);
    }

    fn connect(ip_port: &str, agent: &str, timeout: u64) -> TcpStream {
        loop {
            let host: String = ip_port.into();
            let error = match TcpStream::connect(host) {
                Ok(stream) => {
                    log(ip_port, agent, "Connected");
                    stream
                        .set_read_timeout(Some(time::Duration::new(
                            timeout, 0,
                        )))
                        .expect("Set Socket Read Timeout");
                    stream
                        .set_write_timeout(Some(time::Duration::new(
                            timeout, 0,
                        )))
                        .expect("Set Socket Write Timeout");
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
        let mut socket = Socket::new(host, "HTTP Agent", 5);

        let request = "GET / HTTP/1.1\r\nHost: pubnub.com\r\n\r\n";
        let _ = socket.write(request).expect("data written");
    }

    #[test]
    fn read_ok() {
        let host = "www.pubnub.com:80".into();
        let mut socket = Socket::new(host, "HTTP Agent", 5);

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
