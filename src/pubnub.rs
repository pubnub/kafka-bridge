use crate::socket::{self, Socket};
use json::object;

pub struct Client {
    socket: Socket<Policy>,
    timetoken: String,
    channel: String,
    publish_key: String,
    subscribe_key: String,
    _secret_key: String,
    agent: String,
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
        self.log("PubNub Initializing...");
    }
    fn connected(&self) {
        self.log("PubNub Connected Successfully");
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

impl Client {
    pub fn new(
        host: &str,
        channel: &str,
        publish_key: &str,
        subscribe_key: &str,
        secret_key: &str,
    ) -> Self {
        let policy = Policy::new(host.into());
        let socket = Socket::new(policy);

        let mut pubnub = Self {
            socket: socket,
            channel: channel.into(),
            timetoken: "0".into(),
            publish_key: publish_key.into(),
            subscribe_key: subscribe_key.into(),
            _secret_key: secret_key.into(),
            agent: "nats-bridge".into(),
        };

        pubnub.subscribe();
        pubnub
    }

    pub fn publish(&mut self, channel: &str, message: &str) {
        let sub_cmd = self.subscribe_command();
        let json_message = json::stringify(message);
        let uri = format!(
            "/publish/{}/{}/0/{}/0/{}?pnsdk={}",
            self.publish_key, self.subscribe_key, channel, json_message, self.agent
        );

        let request = format!("GET {} HTTP/1.1\r\nHost: pubnub\r\n\r\n", uri);
        self.socket.write(&request, &sub_cmd);

        // TODO Capture Response Code
        loop {
            let line = self.socket.readln(&sub_cmd);
            if line.ok && line.data.len() == 2 {
                break;
            }
        }
    }

    fn subscribe(&mut self) {
        if self.channel.len() <= 0 { return }
        let sub_cmd = self.subscribe_command();
        self.socket.write(&sub_cmd, &"");
    }

    fn subscribe_command(&mut self) -> String {
        if self.channel.len() <= 0 { "".into() }
        else {
            format!(
                "SUB {channel}{timetoken}\r\n",
                channel=self.channel,
                timetoken=self.timetoken,
            )
        }
    }
}
