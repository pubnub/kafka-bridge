// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Imports
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
use crate::socket::{Socket, SocketPolicy};
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
struct NATSSocketPolicy {
    channel: String,
    host: String,
    client_id: u64,
}
impl SocketPolicy for NATSSocketPolicy {
    // Socket Events
    fn initialized(&self) {
        self.log("NATS Initailzield");
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
    fn connect_after_initialized(&self) -> bool {
        true
    }
    fn data_on_connect(&self) -> String {
        format!("SUB {} {}\r\n", self.channel, self.client_id)
    }
    fn reconnect_after_disconnected(&self) -> bool {
        true
    }
    fn retry_when_unreachable(&self) -> bool {
        true
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
    pub fn new(host: &str, channel: &str) -> Self {
        let policy = NATSSocketPolicy {
            host: host.into(),
            channel: channel.into(),
            client_id: 1, // TODO get ClientID
        };
        let mut socket = Socket::new("NATS", host.into(), policy);

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

/*
impl Drop for NATS {
    fn drop(&mut self) {
        self.socket.disconnect().expect("Failed to disconnect NATS during drop");
    }
}
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
