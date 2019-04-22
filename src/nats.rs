// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Imports
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
use crate::socket::{Socket, SocketPolicy, SocketConnectivityPolicy};

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// NATS Struct
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
pub(crate) struct NATS {
    pub(crate) socket: Socket,
    pub(crate) channel: &'static str,
}

impl NATS {
    pub fn new(
        host: &'static str,
        channel: &'static str,
    ) -> Self {
        let policy = SocketPolicy {
            connected: &Self::connected,
        };

        let mut socket = Socket::new("NATS", host, policy);
        let mut nats = Self {socket: socket, channel: channel};

        nats.socket.connect();

        nats
    }
}

impl SocketConnectivityPolicy for NATS {
    fn connected(&self) {
        println!("{} Connected!", self.socket.name);
    }
}
