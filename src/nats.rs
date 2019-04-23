// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Imports
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
use crate::socket::{Socket, SocketPolicy};

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// NATS User Structs
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
pub(crate) struct NATS {
    pub(crate) channel: String,
    pub(crate) socket: Socket,
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
struct NATSReconnectPolicy { client_id: u64 }
impl SocketPolicy for NATSReconnectPolicy {
    fn initialized(&self, mut socket: &Socket) {
        println!("Initailzield ! ( SocketPolicy ) {}", socket.name);
        // TODO socket.connect ?
    }
    fn connected(&self, mut socket: &Socket) {
        println!("Connected ! ( SocketPolicy )");
    }
    fn disconnected(&self, mut socket: &Socket) {}
    fn unreachable(&self, mut socket: &Socket) {
        println!("Unreachable Host! ( SocketPolicy )");
    }
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// NATS
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
impl NATS {
    pub fn new(host: &str, channel: &str) -> Self {
        let policy = NATSReconnectPolicy { client_id: 0 };
        let mut socket = Socket::new("NATS", host.into(), policy);
        //socket.set_policy(AutoReconnectPolicy);
        Self {
            channel: channel.into(),
            socket: socket,
            //_authkey: authkey.into(),
            //_user: user.into(),
            //_password: password.into(),
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
impl HasSocketPolicy for NATS {
    fn connected(&mut self) {
        println!("{} Connected!", self.socket.name);
        self.subscribe();
    }
    fn disconnected(&mut self) {
        println!("{} Disconnected!", self.socket.name);
        println!("{} Reconnecting...", self.socket.name);
        // self.socket.connect();
        self.subscribe();
    }
}
*/

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
