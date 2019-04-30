use crate::socket::{self, Socket};
use json::object;

pub struct Message {
    pub channel: String,
    pub my_id: String,
    pub sender_id: String,
    pub data: String,
    pub ok: bool,
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

/*
impl From <socket::Error> for Error {
    fn from (error: socket::Error) -> Error {
        Error::Socket(error)
    }
}
*/

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/// # NATS Subscribe Client
///
/// This client lib offers subscribe support to NATS.
///
/// ```no_run
/// use nats_bridge::nats::SubscribeClient;
///
/// let mut nats = SubscribeClient::new("0.0.0.0:4222", "my_channel");
///
/// loop {
///     let message = nats.next_message();
///     assert!(message.is_ok());
///     println!("{} -> {}", message.channel, message.data);
/// }
/// ```
///
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
impl SubscribeClient {
    pub fn new(
        host: &str,
        channel: &str,
    ) -> Result<Self, Error> {
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
    /// let mut nats = SubscribeClient::new("0.0.0.0:4222", channel);
    ///
    /// let message = nats.next_message();
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
    /// let mut nats = SubscribeClient::new("0.0.0.0:4222", channel);
    ///
    /// let message = nats.next_message();
    /// ```
    pub fn next_message(&mut self) -> Message {
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

                    return Message {
                        channel: detail[1].into(),
                        my_id: detail[2].into(),
                        sender_id: detail[3].into(),
                        data: data.trim().into(),
                        ok: true,
                    };
                },
                _ => continue,
            }
        }
    }

    #[cfg(test)]
    pub fn ping(&mut self) -> Result<String, Error> {
        self.socket.write("PING\r\n");
        match self.socket.readln("") {
            Ok(data) => Ok(data),
            Error(error) => Err(Error::Ping),
        }
    }

    #[cfg(test)]
    pub fn exit(&mut self) -> Result<(), Error> {
        match self.socket.write("EXIT\r\n") {
            Ok(data) => Ok(()),
            Error(error) => Err(Error::Exit),
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
/// let mut nats = PublishClient::new("0.0.0.0:4222");
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
    /// use nats_bridge::nats::Client;
    ///
    /// let mut nats = Client::new("0.0.0.0:4222", "channel");
    /// let channel = "demo";
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
    pub fn ping(&mut self) -> String {
        let sub_cmd = self.subscribe_string();
        self.socket.write("PING\r\n", &sub_cmd);
        let ok = self.socket.readln("");
        ok.data
    }

    #[cfg(test)]
    pub fn exit(&mut self) {
        let sub_cmd = self.subscribe_string();
        self.socket.write("EXIT\r\n", &sub_cmd);
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
                    socket.write_all(b"INFO {\"server_id\":\"asbLGfs3r7pgZwucUxYnPn\",\"version\":\"1.4.1\",\"proto\":1,\"git_commit\":\"3e64f0b\",\"go\":\"go1.11.5\",\"host\":\"0.0.0.0\",\"port\":4222,\"max_payload\":1048576,\"client_id\":94295}\r\n").expect("Could not send info");

                    let mut reader =
                        BufReader::new(socket.try_clone().expect("Unable to clone socket"));
                    let mut line = String::new();

                    loop {
                        line.clear();
                        let size = reader.read_line(&mut line).expect("Unable to read line");
                        if size == 0 {
                            eprintln!("Socket disconnected while reading");
                            break;
                        }

                        match line.as_ref() {
                            "EXIT\r\n" => break,
                            "PING\r\n" => {
                                socket.write_all(b"PONG\r\n").expect("Unable to write");
                            }
                            "SUB demo 1\r\n" => continue,
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
    fn subscribe_ok() {
        let host = "0.0.0.0:4221";
        let mock = NATSMock::new(host).expect("Unable to listen");
        let t = thread::spawn(move || {
            mock.process();
        });

        let channel = "demo";
        let mut nats = Client::new(host, channel);

        nats.publish(channel, "Hello");
        let message = nats.next_message();
        assert!(message.ok);

        nats.exit();
        t.join().expect("Thread died early...");
    }

    #[test]
    fn ping_ok() {
        let channel = "demo";
        let host = "0.0.0.0:4223";
        let mock = NATSMock::new(host).expect("Unable to listen");
        let t = thread::spawn(move || {
            mock.process();
        });

        let mut nats = Client::new(host, channel);

        let pong = nats.ping();
        assert_eq!(pong, "PONG\r\n");

        nats.exit();
        t.join().expect("Thread died early...");
    }
}
