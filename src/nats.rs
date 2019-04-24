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
    policy: NATSSocketPolicy,
    socket: Socket,
}
pub(crate) struct NATSMessage {
    pub(crate) channel: String,
    pub(crate) my_id: String,
    pub(crate) sender_id: String,
    pub(crate) data: String,
    pub(crate) ok: bool,
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
    fn initializing(&self) { self.log("NATS Initializing..."); }
    fn connected(&self) { self.log("NATS Connected Successfully"); }
    fn disconnected(&self, error: &str) { self.log(error); }
    fn unreachable(&self, error: &str) { self.log(error); }

    // Socket Behaviors
    fn data_on_connect(&self) -> String {
        format!("SUB {} {}\r\n", self.channel, self.client_id)
    }
    fn retry_delay_after_disconnected(&self) -> u64 { 1 }
    fn retry_delay_when_unreachable(&self) -> u64 { 1 }
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
/// # NATS
/// 
/// This client lib offers *durable* publish and subscribe support to NATS.
/// 
/// ```
/// let mut nats = nats::NATS::new("0.0.0.0:4222", "demo");
/// 
/// loop {
///     let message = nats.next_message();
///     assert!(message.ok);
///     println!("{} -> {}", message.channel, message.data);
/// }
/// ```
/// 
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
impl NATS {
    pub fn new(host: &'static str, channel: &'static str) -> Self {
        let policy = NATSSocketPolicy {
            host: host.into(),
            channel: channel.into(),
            client_id: 1, // TODO get ClientID from `info`
        };
        let mut socket = Socket::new("NATS", policy);
        let natspolicy = policy;
        let info = socket.readln();
        // TODO use info

        Self {
            channel: channel.into(),
            policy: natspolicy,
            socket: socket,
        }
    }

    pub fn publish(&mut self, channel: &str, data: &str) {
        self.socket.write(&format!(
            "PUB {channel} {length}\r\n{data}\r\n",
            channel=channel,
            length=data.chars().count(),
            data=data,
        ));
    }

    // TODO VEC
    pub fn next_message(&mut self) -> NATSMessage {
        loop {
            let line = self.socket.readln();
            if line.size <= 0 { continue; }

            let mut detail = line.data.split_whitespace();
            let command = detail.next();
            if Some("PING") == command { self.socket.write("PONG\r\n"); }
            if Some("MSG") != command { continue; }

            let line = self.socket.readln();
            if !line.ok { continue; }

            // TODO Check length of detail iterator
            // TODO vecotr collection dealio
            break NATSMessage {
                channel: detail.next().unwrap().into(),
                my_id: detail.next().unwrap().into(),
                sender_id: detail.next().unwrap().into(),
                data: line.data.trim().into(),
                ok: true,
            };
        }
    }

    #[cfg(test)]
    pub fn ping(&mut self) -> String {
        self.socket.write(&format!("PING"));
        let ok = self.socket.readln();
        ok.data
    }
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Drop interface
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
impl Drop for NATS {
    fn drop(&mut self) {
        self.socket.disconnect();
    }
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Tests
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_ok() {
        let channel = "demo";
        let host = "0.0.0.0:4222";
        let nats = NATS::new(host, channel);

        assert!(nats.socket.host == host);
        assert!(nats.channel == channel);
    }

    #[test]
    fn subscribe_ok() {
        let channel = "demo";
        let host = "0.0.0.0:4222";
        let mut nats = NATS::new(host, channel);

        assert!(nats.socket.host == host);
        assert!(nats.channel == channel);

        nats.publish(channel, "Hello");
        let message = nats.next_message();
        assert!(message.ok);
    }

    #[test]
    fn ping_ok() {
        let channel = "demo";
        let host = "0.0.0.0:4222";
        let mut nats = NATS::new(host, channel);

        let pong = nats.ping();
        assert!(pong == "+OK\r\n");
    }
}
