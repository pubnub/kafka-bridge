// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Imports
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
use crate::socket::Socket;

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// NATS
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
pub struct NATS {
    socket: Socket,
    channel: String,
    _authkey: String,
    _user: String,
    _password: String,
}

pub struct NATSMessage {
    pub channel: String,
    pub my_id: String,
    pub sender_id: String,
    pub data: String,
}

impl NATS {
    pub fn new(
        host: &str,
        channel: &str,
        authkey: &str,
        user: &str,
        password: &str,
    ) -> Self {
        let socket = Socket::new("NATS", host);

        let mut nats = NATS {
            socket: socket,
            channel: channel.into(),
            _authkey: authkey.into(),
            _user: user.into(),
            _password: password.into(),
        };

        nats.subscribe();

        nats
    }

    fn subscribe(&mut self) {
        let subscription = format!("SUB {} 1\r\n", self.channel);
        self.socket.write(&subscription).expect("Unable to write to NATS socket");
    }

    pub fn next_message(&mut self) -> Result<NATSMessage, std::io::Error> {
        Ok(loop {

            // create socket lib that is durable and implemetns the common
            // read/write and reconnect on errors.
            let line = self.socket.read_line();

            if line.size <= 0 { continue; }

            let mut detail = line.data.split_whitespace();
            if Some("MSG") != detail.next() { continue; }

            let line = self.socket.read_line();
            if !line.ok { continue; }

            // TODO Check length of detail iterator
            // TODO vecotr collection
            break NATSMessage {
                channel: detail.next().unwrap().into(),
                my_id: detail.next().unwrap().into(),
                sender_id: detail.next().unwrap().into(),
                data: line.data.trim().into(),
            };
        })
    }

    #[cfg(test)]
    pub fn ping(&mut self) -> String {
        let ping = format!("PING");
        let _ = self.socket.write(&ping);

        let line = self.socket.read_line();
        line.data
    }
}

impl Drop for NATS {
    fn drop(&mut self) {
        self.socket.disconnect().expect("Failed to disconnect NATS during drop");
    }
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Tests
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_ok() {
        let host = "0.0.0.0:4222";
        let nats = NATS::new(host, "demo", "", "", "");
        assert!(nats.socket.client.host == host);
    }

    #[test]
    fn ping_ok() {
        let host = "0.0.0.0:4222";
        let mut nats = NATS::new(host, "demo", "", "", "");
        let pong = nats.ping();
        assert!(pong == "PONG");
    }
}
