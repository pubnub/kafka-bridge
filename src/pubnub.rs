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
        Self { 
            host: host.into(),
        }
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
        let mut socket = Socket::new(policy);

        Self {
            socket: socket,
            channel: "0".into(),
            timetoken: "0".into(),
        }
    /*
        let policy = SocketPolicy {
            connected: &Self::connected,
        };
        let mut socket = Socket::new("PubNub", host, policy);
        let mut pubnub = Self {socket: socket, channel: channel};

        pubnub.socket.connect();

        pubnub
        */
    }
}

/*
impl SocketConnectivityPolicy for PubNub {
    fn connected(&self) {
        println!("{} Connected!", self.socket.name);
    }
}
*/
