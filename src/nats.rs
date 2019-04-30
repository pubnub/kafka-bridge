use crate::socket::{self, Socket};
use json::object;

pub struct Message {
    pub channel: String,
    pub my_id: String,
    pub sender_id: String,
    pub data: String,
}

struct Policy {
    host: String,
}

impl socket::Policy for Policy {
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
    fn unwritable(&self, error: &str) {
        self.log(error);
    }
}

impl Policy {
    fn new(host: &str) -> Self {
        Self { host: host.into() }
    }

    fn log(&self, message: &str) {
        println!(
            "{}",
            json::stringify(object! {
                "message" => message,
                "client" => "NATS",
                "host" => self.host.clone(),
            })
        );
    }
}

#[derive(Debug)]
pub enum Error {
    Initialize,
    Publish,
    Subscribe,
    Ping,
    Exit,
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/// # NATS Subscribe Client
///
/// This client lib offers subscribe support to NATS.
///
/// ```no_run
/// use nats_bridge::nats::SubscribeClient;
///
/// let channel = "demo";
/// let mut nats = SubscribeClient::new("0.0.0.0:4222", channel)
///     .expect("NATS Subscribe Client");
///
/// let result = nats.next_message();
/// assert!(result.is_ok());
/// let message = result.expect("Received Message");
/// println!("{} -> {}", message.channel, message.data);
/// ```
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
impl SubscribeClient {
    pub fn new(host: &str, channel: &str) -> Result<Self, Error> {
        let policy = Policy::new(host);
        let mut socket = Socket::new(policy);

        // Get Client ID
        let infoln = match socket.readln() {
            Ok(data) => data,
            Err(_) => return Err(Error::Initialize),
        };
        let json = infoln
            .trim()
            .split_whitespace()
            .skip(1)
            .next()
            .expect("NATS info missing JSON");

        let json_info = json::parse(json).expect("NATS info JSON on Connect");
        let client_id = json_info["client_id"].to_string();

        let mut nats = Self {
            socket: socket,
            client_id: client_id,
            channel: channel.into(),
        };

        nats.subscribe();
        Ok(nats)
    }

    /// ## Receive NATS Messages
    ///
    /// Subscribe to any NATS channel.
    /// > Warning: This method can only be called once per client because
    /// > NATS does not support multiplexing.
    /// > If you need multiple channels, initialize one NATS client per
    /// > channel and put each client into a thread.
    ///
    /// ```no_run
    /// use nats_bridge::nats::SubscribeClient;
    ///
    /// let channel = "demo";
    /// let mut nats = SubscribeClient::new("0.0.0.0:4222", channel)
    ///     .expect("NATS Subscribe Client");
    ///
    /// let message = nats.next_message().expect("Received Message");
    /// ```
    fn subscribe(&mut self) {
        loop {
            let sub = &format!(
                "SUB {channel} {client_id}\r\n",
                channel=self.channel,
                client_id=self.client_id,
            );
            match self.socket.write(sub) {
                Ok(_) => { break; },
                Err(_) => {
                    self.socket.reconnect();
                    self.subscribe();
                },
            };
        }
    }

    /// ## Receive NATS Messages
    ///
    /// Easy way to get messages from the initialized channel.
    ///
    /// ```no_run
    /// use nats_bridge::nats::SubscribeClient;
    ///
    /// let channel = "demo";
    /// let mut nats = SubscribeClient::new("0.0.0.0:4222", channel)
    ///     .expect("NATS Subscribe Client");
    ///
    /// let message = nats.next_message().expect("Received Message");
    /// ```
    pub fn next_message(&mut self) -> Result<Message, Error> {
        loop {
            let result = self.socket.readln();
            if result.is_err() {
                self.socket.reconnect();
                self.subscribe();
                continue;
            }
            let data = result.expect("NATS Socket Read");

            let detail: Vec<_> = data.trim().split_whitespace().collect();
            if detail.is_empty() {
                continue;
            }

            let command = detail[0];
            match command {
                "PING" => {
                    match self.socket.write("PONG\r\n") {
                        Ok(_) => { },
                        Err(_) => {
                            self.socket.reconnect();
                            self.subscribe();
                        },
                    };
                },
                "MSG" => {
                    if detail.len() != 4 {
                        continue;
                    }

                    let result = self.socket.readln();
                    if result.is_err() {
                        self.socket.reconnect();
                        self.subscribe();
                        continue;
                    }

                    return Ok(Message {
                        channel: detail[1].into(),
                        my_id: detail[2].into(),
                        sender_id: detail[3].into(),
                        data: data.trim().into(),
                    });
                },
                _ => continue,
            }
        }
    }

    #[cfg(test)]
    pub fn ping(&mut self) -> Result<String, Error> {
        let _size = match self.socket.write("PING\r\n") {
            Ok(size) => size,
            Err(_error) => return Err(Error::Ping),
        };
        match self.socket.readln() {
            Ok(data) => Ok(data),
            Err(_error) => Err(Error::Ping),
        }
    }

    #[cfg(test)]
    pub fn exit(&mut self) -> Result<(), Error> {
        match self.socket.write("EXIT\r\n") {
            Ok(_size) => Ok(()),
            Err(_error) => Err(Error::Exit),
        }
    }
}

pub struct SubscribeClient {
    socket: Socket<Policy>,
    client_id: String,
    channel: String,
}

impl Drop for SubscribeClient {
    fn drop(&mut self) {
        self.socket.disconnect();
    }
}


// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/// # NATS Publish Client
///
/// This client lib offers publish support to NATS.
///
/// ```no_run
/// use nats_bridge::nats::PublishClient;
///
/// let mut nats = PublishClient::new("0.0.0.0:4222").expect("NATS PUB");
///
/// loop {
///     let result = nats.publish("hello", "channel");
///     assert!(result.is_ok());
/// }
/// ```
///
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
impl PublishClient {
    pub fn new(host: &str) -> Result<Self, Error> {
        let policy = Policy::new(host);
        let mut socket = Socket::new(policy);

        // Get Client ID
        let infoln = match socket.readln() {
            Ok(data) => data,
            Err(_) => return Err(Error::Initialize),
        };
        let json = infoln
            .trim()
            .split_whitespace()
            .skip(1)
            .next()
            .expect("NATS info missing JSON");

        let json_info = json::parse(json).expect("NATS info JSON on Connect");
        let client_id = json_info["client_id"].to_string();

        let pubnub = Self {
            socket: socket,
            _client_id: client_id,
        };

        Ok(pubnub)
    }

    /// ## Send NATS Messages
    ///
    /// Easy way to send messages to any NATS channel.
    ///
    /// ```no_run
    /// use nats_bridge::nats::PublishClient;
    ///
    /// let channel = "demo";
    /// let mut nats = PublishClient::new("0.0.0.0:4222")
    ///     .expect("NATS Publish Client");
    ///
    /// nats.publish(channel, "Hello").expect("publish sent");
    /// ```
    pub fn publish(&mut self, channel: &str, data: &str) -> Result<(), Error> {
        let pubcmd = &format!(
            "PUB {channel} {length}\r\n{data}\r\n",
            channel = channel,
            length = data.len(),
            data = data,
        );
        match self.socket.write(pubcmd) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::Publish),
        }
    }

    #[cfg(test)]
    pub fn ping(&mut self) -> Result<String, Error> {
        let _size = match self.socket.write("PING\r\n") {
            Ok(size) => size,
            Err(_error) => return Err(Error::Ping),
        };
        match self.socket.readln() {
            Ok(data) => Ok(data),
            Err(_error) => Err(Error::Ping),
        }
    }

    #[cfg(test)]
    pub fn exit(&mut self) -> Result<(), Error> {
        match self.socket.write("EXIT\r\n") {
            Ok(_size) => Ok(()),
            Err(_error) => Err(Error::Exit),
        }
    }
}

pub struct PublishClient {
    socket: Socket<Policy>,
    _client_id: String,
}

impl Drop for PublishClient {
    fn drop(&mut self) {
        self.socket.disconnect();
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;
    use std::thread;

    struct NATSMock {
        listener: TcpListener,
    }

    impl NATSMock {
        fn new(host: &str) -> std::io::Result<Self> {
            let listener = TcpListener::bind(host)?;
            Ok(Self { listener })
        }

        fn process(&self) {
            match self.listener.accept() {
                Ok((mut socket, _addr)) => {
                    socket.write_all(b"INFO {\"server_id\":\"asbLGfs3r7pgZwucUxYnPn\",\"version\":\"1.4.1\",\"proto\":1,\"git_commit\":\"3e64f0b\",\"go\":\"go1.11.5\",\"host\":\"0.0.0.0\",\"port\":4222,\"max_payload\":1048576,\"client_id\":9999}\r\n").expect("Could not send info");

                    let mut reader =
                        BufReader::new(socket.try_clone()
                            .expect("Unable to clone socket"));
                    let mut line = String::new();

                    loop {
                        line.clear();
                        let size = reader.read_line(&mut line)
                            .expect("Unable to read line");
                        if size == 0 {
                            eprintln!("Socket disconnected while reading");
                            break;
                        }

                        match line.as_ref() {
                            "EXIT\r\n" => break,
                            "PING\r\n" => {
                                socket.write_all(b"PONG\r\n").expect("Unable to write");
                            }
                            "SUB demo 9999\r\n" => {
                                socket.write_all(b"MSG demo 9999 5\r\nKNOCK\r\n")
                                    .expect("Unable to write");
                            },
                            "PUB demo 5\r\n" => {
                                line.clear();
                                reader.read_line(&mut line).expect("Unable to read line");

                                let cmd = format!("MSG demo 1 1\r\n{}", line);
                                socket.write_all(cmd.as_bytes()).expect("Unable to write");
                            }
                            _ => eprintln!("Unexpected line: `{}`", line),
                        };
                    }
                }
                Err(e) => eprintln!("couldn't get client: {:?}", e),
            }
        }
    }

    #[test]
    fn publish_ok() {
        let host = "0.0.0.0:4220";
        let mock = NATSMock::new(host).expect("Unable to listen");
        let t = thread::spawn(move || {
            mock.process();
        });

        let channel = "demo";
        let mut publisher = PublishClient::new(host).expect("NATS Publish Client");

        publisher.publish(channel, "Hello").expect("Message Sent");
        publisher.exit().expect("NATS Connection Closed");
        t.join().expect("Mock TcpStream server");
    }

    #[test]
    fn subscribe_ok() {
        let host = "0.0.0.0:4221";
        let mock = NATSMock::new(host).expect("Unable to listen");
        let t = thread::spawn(move || {
            mock.process();
        });

        let channel = "demo";
        let mut subscriber = SubscribeClient::new(host, channel)
            .expect("NATS Subscribe Client");
        let result = subscriber.next_message();
        assert!(result.is_ok());
        let message = result.expect("Received Message");
        assert!(message.channel.len() > 0);
        subscriber.exit().expect("NATS Socket Closed");
        t.join().expect("Mock TcpStream server");
    }

    #[test]
    fn ping_ok() {
        let host = "0.0.0.0:4223";
        let mock = NATSMock::new(host).expect("Unable to listen");
        let t = thread::spawn(move || {
            mock.process();
        });

        let mut nats = PublishClient::new(host).expect("NATS Publish Client");

        let pong = nats.ping().expect("Pong from Ping");
        assert_eq!(pong, "PONG\r\n");

        nats.exit().expect("NATS Connection Closed");
        t.join().expect("Thread died early...");
    }
}
