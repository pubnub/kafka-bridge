use crate::socket::{self, Socket};
use json::object;

pub struct Client {
    socket: Socket<Policy>,
    client_id: String,
    channel: String,
}

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
    fn initializing(&self) {
        self.log("NATS Initializing...");
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

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/// # NATS Client
///
/// This client lib offers *durable* publish and subscribe support to NATS.
///
/// ```no_run
/// use nats_bridge::nats::Client;
///
/// let mut nats = Client::new("0.0.0.0:4222", "my_channel");
///
/// loop {
///     let message = nats.next_message();
///     assert!(message.ok);
///     println!("{} -> {}", message.channel, message.data);
/// }
/// ```
///
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
impl Client {
    pub fn new(
        host: &str,
        channel: &str,
    ) -> Self {
        let policy = Policy::new(host);
        let mut socket = Socket::new(policy);

        // Get Client ID
        let infoln = socket.readln(&"");
        let details: Vec<_> = infoln.data.trim().split_whitespace().collect();
        assert!(!details.is_empty());
        let info_json = json::parse(&details[1]).expect("Info JSON from NATS");
        let client_id = info_json["client_id"].to_string();

        let mut nats = Self {
            socket: socket,
            client_id: client_id,
            channel: channel.into(),
        };

        nats.subscribe();

        nats
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
    /// nats.publish(channel, "Hello");
    /// ```
    pub fn publish(&mut self, channel: &str, data: &str) {
        let sub_cmd = self.subscribe_command();
        self.socket.write(&format!(
            "PUB {channel} {length}\r\n{data}\r\n",
            channel = channel,
            length = data.len(),
            data = data,
        ), &sub_cmd);
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
    /// use nats_bridge::nats::Client;
    ///
    /// let mut nats = Client::new("0.0.0.0:4222", "channel");
    /// let channel = "demo";
    ///
    /// nats.subscribe();
    /// let message = nats.next_message();
    /// ```
    fn subscribe(&mut self) {
        let sub_cmd = self.subscribe_command();
        self.socket.write(&sub_cmd, &"");
    }

    fn subscribe_command(&mut self) -> String {
        format!(
            "SUB {channel} {client_id}\r\n",
            channel=self.channel,
            client_id=self.client_id,
        )
    }

    /// ## Receive NATS Messages
    ///
    /// Easy way to get messages from the initialized channel.
    ///
    /// ```no_run
    /// use nats_bridge::nats::Client;
    ///
    /// let mut nats = Client::new("0.0.0.0:4222", "channel");
    ///
    /// let channel = "demo";
    /// nats.publish(channel, "Hello");
    ///
    /// let message = nats.next_message();
    /// assert!(message.ok);
    /// assert_eq!(message.data, "Hello");
    /// println!("{}", message.data);
    /// ```
    pub fn next_message(&mut self) -> Message {
        let sub_cmd = self.subscribe_command();
        loop {
            let line = self.socket.readln(&sub_cmd);
            if !line.ok {
                self.subscribe();
            }

            let detail: Vec<_> = line.data.trim().split_whitespace().collect();
            if detail.is_empty() {
                continue;
            }

            let command = detail[0];
            match command {
                "PING" => {
                    let sub_cmd = self.subscribe_command();
                    self.socket.write("PONG\r\n", &sub_cmd);
                },
                "MSG" => {
                    if detail.len() != 4 {
                        continue;
                    }

                    let line = self.socket.readln(&sub_cmd);
                    if !line.ok {
                        continue;
                    }

                    return Message {
                        channel: detail[1].into(),
                        my_id: detail[2].into(),
                        sender_id: detail[3].into(),
                        data: line.data.trim().into(),
                        ok: true,
                    };
                },
                _ => continue,
            }
        }
    }

    #[cfg(test)]
    pub fn ping(&mut self) -> String {
        let sub_cmd = self.subscribe_command();
        self.socket.write("PING\r\n", &sub_cmd);
        let ok = self.socket.readln(&"");
        ok.data
    }

    #[cfg(test)]
    pub fn exit(&mut self) {
        let sub_cmd = self.subscribe_command();
        self.socket.write("EXIT\r\n", &sub_cmd);
    }
}

impl Drop for Client {
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
        //`j.let message = nats.next_message();
        //`j.assert!(message.ok);

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

        //let pong = nats.ping();
        //assert_eq!(pong, "PONG\r\n");

        nats.exit();
        t.join().expect("Thread died early...");
    }
}
