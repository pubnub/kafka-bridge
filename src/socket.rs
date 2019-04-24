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
/// The `impl` Struct is where you store state for the socket connection.
/// 
/// ```
/// use socket::{Socket, SocketPolicy, Line};
/// 
/// #[derive(Copy, Clone)]
/// struct MySocketPolicy {
///     channel: &'static str,
///     host: &'static str,
///     client_id: u64,
///     other_thing: u64,
/// }
/// 
/// impl SocketPolicy for MySocketPolicy {
///     // Socket Attributes
///     fn host(&self) -> &str { &self.host }
/// 
///     // Socket Events
///     fn initializing(&self) { self.log("NATS Initializing"); }
///     fn connected(&self) { self.log("NATS Connected Successfully"); }
///     fn disconnected(&self, error: &str) { self.log(error); }
///     fn unreachable(&self, error: &str) { self.log(error); }
/// 
///     // Socket Behaviors
///     fn data_on_connect(&self) -> String { format!("SUB chan 1\r\n") }
///     fn retry_delay_after_disconnected(&self) -> u64 { 1 }
///     fn retry_delay_when_unreachable(&self) -> u64 { 1 }
/// }
/// ```
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
pub trait SocketPolicy {
    // Attributes
    fn host(&self) -> &str;

    // Events
    fn initializing(&self) {}
    fn connected(&self) {}
    fn disconnected(&self, error: &str) {}
    fn unreachable(&self, error: &str) {}

    // Behaviors
    fn data_on_connect(&self) -> String;
    fn retry_delay_after_disconnected(&self) -> u64;
    fn retry_delay_when_unreachable(&self) -> u64;
}

pub struct Socket {
    pub client: String,
    pub host: String,
    policy: Box<SocketPolicy>,
    stream: TcpStream,
    reader: BufReader<TcpStream>,
}

pub struct Line {
    pub ok: bool,
    pub size: usize,
    pub data: String,
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/// # Socket
///
/// The user interface for this library.
///
/// ```
/// pub struct MyClient {
///     pub channel: String,
///     socket: Socket,
/// }
/// impl MyClient {
///     pub fn new(host: &'static str, channel: &'static str) -> Self {
///         let policy = MySocketPolicy {
///             host: host.into(),
///             channel: channel.into(),
///             client_id: 1,
///         };
///         let socket = Socket::new("MyClient", policy);
/// 
///         Self {
///             channel: channel.into(),
///             socket: socket,
///         }
///     }
/// }
/// ```
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
impl Socket {
    pub fn new<P: SocketPolicy + 'static + Copy>(
        client: &str,
        policy: P,
    ) -> Self {
        policy.initializing();
        let stream = Self::connect(&policy);
        policy.connected();

        Self {
            client: client.into(),
            host: policy.host().into(),
            policy: Box::new(policy),
            stream: stream.try_clone().expect("TcpStream"),
            reader: BufReader::new(stream.try_clone().expect("TcpStream")),
        }
    }

    /// ## Write Data
    ///
    /// Write string data to the stream.
    ///
    /// ```
    /// let request = "GET / HTTP/1.1\r\nHost: pubnub.com\r\n\r\n";
    /// socket.write(&request);
    /// 
    /// let line = socket.readln();
    /// assert!(line.ok);
    /// assert!(line.size > 0);
    /// ```
    pub fn write(&mut self, data: &str) {
        loop {
            let result = self.stream.write(data.as_bytes());
            match result {
                Ok(size) => break,
                Err(error) => {
                    self.policy.disconnected(&format!("{}",error));
                    self.reconnect();
                }
            }
        }
    }

    /// ## Read Line
    ///
    /// Read a line of data from the stream.
    ///
    /// ```
    /// let line = socket.readln();
    /// assert!(line.ok);
    /// assert!(line.size > 0);
    /// ```
    pub fn readln(&mut self) -> Line {
        loop {
            let mut line = String::new();
            let result   = self.reader.read_line(&mut line);
            let size     = result.unwrap_or_else( |_| 0 );

            if size == 0 {
                self.reconnect();
                break Line { ok: false, size: size, data: line };
            }

            break Line { ok: true, size: size, data: line };
        }
    }

    /// ## Disconnect
    ///
    /// This will courteously turn off the connection of your socket.
    ///
    /// ```
    /// socket.disconnect();
    /// ```
    pub fn disconnect(&mut self) {
        self.stream.shutdown(Shutdown::Both).expect("Shutdown");
    }

    fn connect<P: SocketPolicy + 'static + ?Sized>(policy: &P) -> TcpStream {
        let retry_delay = policy.retry_delay_when_unreachable();
        loop {
            // Open connection and send initialization data
            let host: String = policy.host().into();
            let connection = TcpStream::connect(host);
            let error = match connection {
                Ok(mut stream) => {
                    let data = policy.data_on_connect();
                    if data.chars().count() > 0 {
                        stream.write(data.as_bytes());
                    }
                    return stream;
                },
                Err(error) => error,
            };

            // Retry connection until the host becomes available
            policy.unreachable(&format!("{}", error));
            thread::sleep(time::Duration::new(retry_delay, 0));
        }
    }

    fn reconnect(&mut self) {
        let retry_delay = self.policy.retry_delay_after_disconnected();
        thread::sleep(time::Duration::new(retry_delay, 0));
        let stream = Self::connect(&*self.policy);
        self.stream = stream.try_clone().expect("TcpStream");
        self.reader = BufReader::new(stream.try_clone().expect("TcpStream"));
    }
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Tests
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
#[cfg(test)]
mod socket_tests {
    use super::*;
    use json::object;

    #[derive(Copy, Clone)]
    struct MySocketPolicy {
        host: &'static str,
    }

    impl SocketPolicy for MySocketPolicy {
        // Socket Attributes
        fn host(&self) -> &str { &self.host }

        // Socket Events
        fn initializing(&self) { self.log("NATS Initializing"); }
        fn connected(&self) { self.log("NATS Connected Successfully"); }
        fn disconnected(&self, error: &str) { self.log(error); }
        fn unreachable(&self, error: &str) { self.log(error); }

        // Socket Behaviors
        fn data_on_connect(&self) -> String { format!("SUB chan 1\r\n") }
        fn retry_delay_after_disconnected(&self) -> u64 { 1 }
        fn retry_delay_when_unreachable(&self) -> u64 { 1 }
    }

    impl MySocketPolicy {
        fn log(&self, message: &str) {
            println!("{}", json::stringify(object!{ 
                "message" => message,
                "client" => "MyClient",
                "host" => self.host.clone(),
            }));
        }
    }

    #[test]
    fn connect_ok() {
        let host = "www.pubnub.com:80";
        let client = "MyClient";
        let policy = MySocketPolicy {
            host: host.into(),
        };
        let socket = Socket::new(client, policy);

        assert!(socket.host == host);
        assert!(socket.client == client);
    }

    #[test]
    fn write_ok() {
        let host = "www.pubnub.com:80";
        let client = "MyClient";
        let policy = MySocketPolicy {
            host: host.into(),
        };
        let mut socket = Socket::new(client, policy);

        let request = "GET / HTTP/1.1\r\nHost: pubnub.com\r\n\r\n";
        socket.write(request);
    }

    #[test]
    fn read_ok() {
        let host = "www.pubnub.com:80";
        let client = "MyClient";
        let policy = MySocketPolicy {
            host: host.into(),
        };
        let mut socket = Socket::new(client, policy);

        let request = "GET / HTTP/1.1\r\nHost: pubnub.com\r\n\r\n";
        socket.write(request);

        let line = socket.readln();
        assert!(line.ok);
        assert!(line.size > 0);

        let line = socket.readln();
        assert!(line.ok);
        assert!(line.size > 0);
    }
}
