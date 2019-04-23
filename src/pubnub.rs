// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Imports
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
use crate::socket::{Socket, SocketPolicy, HasSocketPolicy};

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// PubNub Struct
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
pub(crate) struct PubNub {
    pub(crate) socket: Socket,
    pub(crate) channel: &'static str,
}

impl PubNub {
    pub fn new(
        host: &'static str,
        channel: &'static str,
    ) -> Self {
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
