use crate::socket::{self, Socket};
use json::object;

pub struct Client {
    socket: Socket<Policy>,
    timetoken: String,
    channel: String,
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
    ) -> Self {
        let policy = Policy::new(host.into());
        let socket = Socket::new(policy);

        let mut pubnub = Self {
            socket: socket,
            channel: channel.into(),
            timetoken: "0".into(),
        };

        pubnub
    }

    fn subscribe_command(&mut self) -> String {
        format!(
            "SUB {channel}\r\n",
            channel=self.channel,
        )
    }
}
