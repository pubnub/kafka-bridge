// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Imports
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
use crate::socket::{Socket, SocketPolicy, Line};
use json::object;

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// NATS End-user Interface
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
pub(crate) struct NATS {
    pub(crate) channel: String,
    socket: Socket,
}
pub(crate) struct NATSMessage {
    pub(crate) channel: String,
    pub(crate) my_id: String,
    pub(crate) sender_id: String,
    pub(crate) data: String,
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// NATS Socket Policy ( Wire State & Events )
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
#[derive(Copy, Clone)]
struct NATSSocketPolicy {
    channel: &'static str,
    host: &'static str,
    client_id: u64,
}
impl SocketPolicy for NATSSocketPolicy {
    // Socket Attributes
    fn host(&self) -> &str { &self.host }

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

    // Socket Behaviors
    fn data_on_connect(&self) -> String {
        format!("SUB {} {}\r\n", self.channel, self.client_id)
    }
    fn retry_delay_after_disconnected(&self) -> u64 {
        1
    }
    fn retry_delay_when_unreachable(&self) -> u64 {
        1
    }
}
impl NATSSocketPolicy {
    fn log(&self, message: &str) {
        println!("{}", json::stringify(object!{ 
            "message" => message,
            "client" => "NATS",
            "channel" => self.channel.clone(),
            "host" => self.host.clone(),
        }));
    }
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// NATS
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
impl NATS {
    pub fn new(host: &'static str, channel: &'static str) -> Self {
        let policy = NATSSocketPolicy {
            host: host.into(),
            channel: channel.into(),
            client_id: 1, // TODO get ClientID
        };
        let socket = Socket::new("NATS", policy);

        Self {
            channel: channel.into(),
            socket: socket,
        }
    }

    fn subscribe(&mut self) {
        println!("SUB {} 1\r\n", self.channel);
        //let subscription = format!("SUB {} 1\r\n", self.channel);
        //self.socket.write(&subscription).expect("Unable to write to NATS socket");
    }

    /*
    pub fn next_message(&mut self) -> Result<NATSMessage, std::io::Error> {
        Ok(loop {

            // create socket lib that is durable and implemetns the common
            // read/write and reconnect on errors.
            let line = self.socket.read_line();

            if line.size <= 0 { continue; }

        nats.socket.connect();

        nats
    }
    */
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Drop interface
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
impl Drop for NATS {
    fn drop(&mut self) {
        self.socket.disconnect();
    }
}

/*
*/

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Tests
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_ok() {
        let channel = "demo-channel";
        let host = "0.0.0.0:4222";
        let nats = NATS::new(host, channel);

        assert!(nats.socket.host == host);
        assert!(nats.channel == channel);
    }

    #[test]
    fn subscribe_ok() {
        let channel = "demo-channel";
        let host = "0.0.0.0:4222";
        let mut nats = NATS::new(host, channel);

        nats.subscribe();

        assert!(nats.socket.host == host);
        assert!(nats.channel == channel);
    }


    /*
    #[test]
    fn ping_ok() {
        let host = "0.0.0.0:4222";
        let mut nats = NATS::new(host, "demo");
        //let pong = nats.ping();
        //assert!(pong == "PONG");
    }
    */
}
