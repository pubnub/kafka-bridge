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
///     fn initializing(&self) {
///         self.log("NATS Initializing");
///     }
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
    fn initializing(&self) {
        println!("initializing");
    }
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

pub(crate) struct Socket<P: Policy> {
    policy: P,
    stream: TcpStream,
    reader: BufReader<TcpStream>,
}

pub(crate) struct Line {
    pub(crate) ok: bool,
    pub(crate) data: String,
}

fn connect<P: Policy>(policy: &P) -> TcpStream {
    let retry_delay = policy.retry_delay_when_unreachable();
    loop {
        // Open connection and send initialization data
        let host: String = policy.host().into();
        let error = match TcpStream::connect(host) {
            Ok(stream) => return stream,
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
    pub(crate) fn write(&mut self, data: &str, data_on_reconnect: &str) {
        loop {
            let result = self.stream.write(data.as_bytes());
            match result {
                Ok(size) => {
                    if size > 0 {
                        break;
                    }
                    self.policy.disconnected("No data has been written.");
                    self.reconnect();
                    if data_on_reconnect.len() > 0 {
                        self.write(data_on_reconnect, "");
                    }
                }
                Err(error) => {
                    let error = format!("{}", error);
                    self.policy.unwritable(&error);
                    self.policy.disconnected(&error);
                    self.reconnect();
                    if data_on_reconnect.len() > 0 {
                        self.write(data_on_reconnect, "");
                    }
                }
            };
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
    /// let line = socket.readln(&"");
    /// ```
    pub(crate) fn readln(&mut self, data_on_reconnect: &str) -> Line {
        let mut line = String::new();
        let result = self.reader.read_line(&mut line);
        let size = result.unwrap_or_else(|_| 0);

        if size == 0 {
            self.reconnect();
            if data_on_reconnect.len() > 0 {
                self.write(data_on_reconnect, "");
            }
            return Line {
                ok: false,
                data: line,
            };
        }

        Line {
            ok: true,
            data: line,
        }
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

    fn reconnect(&mut self) {
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
        fn initializing(&self) {
            self.log("NATS Initializing");
        }
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
        socket.write(request, &"");
    }

    #[test]
    fn read_ok() {
        let host = "www.pubnub.com:80".into();
        let policy = MySocketPolicy { host };
        let mut socket = Socket::new(policy);

        let request = "GET / HTTP/1.1\r\nHost: pubnub.com\r\n\r\n";
        socket.write(request, &"");

        let line = socket.readln(&"");
        assert!(line.ok);
        assert!(line.data.len() > 0);

        let line = socket.readln(&"");
        assert!(line.ok);
        assert!(line.data.len() > 0);
    }
}
