use std::io::{BufRead, BufReader, Write};
use std::net::{Shutdown, TcpStream};
use std::{thread, time};

/// # Socket Policy
///
/// Describes what actions a Socket will take for various situations.
/// This is where you store state for the socket connection.
///
/// ```private
/// use nats_bridge::socket::{Socket, Policy, Line};
///
/// struct MySocketPolicy {
///     channel: String,
///     host: String,
///     client_id: u64,
///     other_thing: u64,
/// }
///
/// impl MySocketPolicy {
///     fn log(&self, message: &str) {
///         println!("{}", message);
///     }
/// }
///
/// impl Policy for MySocketPolicy {
///     // Socket Attributes
///     fn host(&self) -> &str {
///         &self.host
///     }
///
///     // Socket Events
///     fn connected(&self) {
///         self.log("NATS Connected Successfully");
///     }
///     fn disconnected(&self, error: &str) {
///         self.log(error);
///     }
///     fn unreachable(&self, error: &str) {
///         self.log(error);
///     }
///     fn unwritable(&self, error: &str) {
///         self.log(error);
///     }
///
///     // Socket Behaviors
///     fn retry_delay_after_disconnected(&self) -> u64 {
///         1
///     }
///     fn retry_delay_when_unreachable(&self) -> u64 {
///         1
///     }
/// }
/// ```
pub(crate) trait Policy {
    // Attributes
    fn host(&self) -> &str;

    // Events
    fn connected(&self) {
        println!("connected");
    }
    fn disconnected(&self, error: &str) {
        eprintln!("{}", error);
    }
    fn unreachable(&self, error: &str) {
        eprintln!("{}", error);
    }
    fn unwritable(&self, error: &str) {
        eprintln!("{}", error);
    }

    // Behaviors
    fn retry_delay_after_disconnected(&self) -> u64 {
        1
    }
    fn retry_delay_when_unreachable(&self) -> u64 {
        1
    }
}

#[derive(Debug)]
pub(crate) enum Error {
    Write,
    Read,
}

/*
impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Write => write!(f, "Write"),
            Error::Read => write!(f, "Read"),
        }
    }
}
*/

pub(crate) struct Socket<P: Policy> {
    policy: P,
    stream: TcpStream,
    reader: BufReader<TcpStream>,
}

fn connect<P: Policy>(policy: &P) -> TcpStream {
    let retry_delay = policy.retry_delay_when_unreachable();
    loop {
        // Open connection and send initialization data
        let host: String = policy.host().into();
        let error = match TcpStream::connect(host) {
            Ok(stream) => {
                policy.connected();
                return stream;
            }
            Err(error) => error,
        };

        // Retry connection until the host becomes available
        policy.unreachable(&format!("{}", error));
        thread::sleep(time::Duration::new(retry_delay, 0));
    }
}

/// # Socket
///
/// The user interface for this library.
///
/// ```private
/// use nats_bridge::socket::{Policy, Socket};
///
/// struct MySocketPolicy {
///     host: String,
/// }
///
/// impl Policy for MySocketPolicy {
///     fn host(&self) -> &str {
///         &self.host
///     }
/// }
///
/// let policy = MySocketPolicy {
///     host: "pubsub.pubnub.com:80".to_string(),
/// };
/// let socket = Socket::new(policy);
/// ```
impl<P: Policy> Socket<P> {
    pub(crate) fn new(policy: P) -> Self {
        let stream = connect(&policy);
        Self {
            policy,
            stream: stream.try_clone().expect("Unable to clone stream"),
            reader: BufReader::new(stream),
        }
    }

    /// ## Write Data
    ///
    /// Write string data to the stream.
    ///
    /// ```private
    /// # use nats_bridge::socket::{Policy, Socket};
    /// # struct MySocketPolicy {
    /// #     host: String,
    /// # }
    /// # impl Policy for MySocketPolicy {
    /// #     fn host(&self) -> &str {
    /// #         &self.host
    /// #     }
    /// # }
    /// # let policy = MySocketPolicy {
    /// #     host: "pubsub.pubnub.com:80".into(),
    /// # };
    /// # let mut socket = Socket::new(policy);
    /// let request = "GET / HTTP/1.1\r\nHost: pubnub.com\r\n\r\n";
    /// socket.write(request);
    /// ```
    pub(crate) fn write(&mut self, data: &str) -> Result<usize, Error> {
        let result = self.stream.write(data.as_bytes());
        match result {
            Ok(size) => {
                if size > 0 {
                    return Ok(size);
                }
                self.policy.disconnected("No data has been written.");
                Err(Error::Write)
            }
            Err(error) => {
                let error = format!("{}", error);
                self.policy.unwritable(&error);
                self.policy.disconnected(&error);
                Err(Error::Write)
            }
        }
    }

    /// ## Read Line
    ///
    /// Read a line of data from the stream.
    ///
    /// ```private
    /// # use nats_bridge::socket::{Policy, Socket};
    /// # struct MySocketPolicy {
    /// #     host: String,
    /// # }
    /// # impl Policy for MySocketPolicy {
    /// #     fn host(&self) -> &str {
    /// #         &self.host
    /// #     }
    /// # }
    /// # let policy = MySocketPolicy {
    /// #     host: "pubsub.pubnub.com:80".into(),
    /// # };
    /// # let mut socket = Socket::new(policy);
    /// # let request = "GET / HTTP/1.1\r\nHost: pubnub.com\r\n\r\n";
    /// # socket.write(request);
    /// let line = socket.readln("");
    /// ```
    pub(crate) fn readln(&mut self) -> Result<String, Error> {
        let mut line = String::new();
        let result = self.reader.read_line(&mut line);
        let size = result.unwrap_or_else(|_| 0);

        if size == 0 {
            Err(Error::Read)?;
        }

        Ok(line)
    }

    /// ## Disconnect
    ///
    /// This will courteously turn off the connection of your socket.
    ///
    /// ```private
    /// # use nats_bridge::socket::{Policy, Socket};
    /// # struct MySocketPolicy {
    /// #     host: String,
    /// # }
    /// # impl Policy for MySocketPolicy {
    /// #     fn host(&self) -> &str {
    /// #         &self.host
    /// #     }
    /// # }
    /// # let policy = MySocketPolicy {
    /// #     host: "pubsub.pubnub.com:80".into(),
    /// # };
    /// # let mut socket = Socket::new(policy);
    /// socket.disconnect();
    /// ```
    pub(crate) fn disconnect(&mut self) {
        self.stream.shutdown(Shutdown::Both).unwrap_or_default();
    }

    pub(crate) fn reconnect(&mut self) {
        let retry_delay = self.policy.retry_delay_after_disconnected();
        thread::sleep(time::Duration::new(retry_delay, 0));
        let stream = connect(&self.policy);
        self.stream = stream.try_clone().expect("Unable to clone stream");
        self.reader = BufReader::new(stream);
    }
}

#[cfg(test)]
mod socket_tests {
    use super::*;
    use json::object;

    struct MySocketPolicy {
        host: String,
    }

    impl Policy for MySocketPolicy {
        // Socket Attributes
        fn host(&self) -> &str {
            &self.host
        }

        // Socket Events
        fn connected(&self) {
            self.log("NATS Connected Successfully");
        }
        fn disconnected(&self, error: &str) {
            self.log(error);
        }
        fn unreachable(&self, error: &str) {
            self.log(error);
        }
    }

    impl MySocketPolicy {
        fn log(&self, message: &str) {
            println!(
                "{}",
                json::stringify(object! {
                    "message" => message,
                    "client" => "MyClient",
                    "host" => self.host.clone(),
                })
            );
        }
    }

    #[test]
    fn write_ok() {
        let host = "www.pubnub.com:80".into();
        let policy = MySocketPolicy { host };
        let mut socket = Socket::new(policy);

        let request = "GET / HTTP/1.1\r\nHost: pubnub.com\r\n\r\n";
        let _ = socket.write(request).expect("data written");
    }

    #[test]
    fn read_ok() {
        let host = "www.pubnub.com:80".into();
        let policy = MySocketPolicy { host };
        let mut socket = Socket::new(policy);

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
